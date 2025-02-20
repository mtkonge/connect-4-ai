use std::collections::HashMap;

use crate::board::{Board, Chip};

#[derive(PartialEq, Clone, Debug)]
pub struct Choice {
    board: Board,
    pub column: usize,
}

impl Choice {
    fn blank() -> Self {
        Self {
            board: Board::new(),
            column: 0,
        }
    }
}

#[repr(transparent)]
struct Weight([i32; Board::COLUMN_LEN]);

impl Weight {
    pub fn blank() -> Self {
        Self([0; Board::COLUMN_LEN])
    }
}

pub struct BotTrainer<'bot> {
    board: Board,
    turn: Chip,
    moves: usize,
    red_bot: &'bot mut Bot,
    yellow_bot: &'bot mut Bot,
}

pub enum GameResult {
    RedWon,
    YellowWon,
    Tie,
}

impl<'bot> BotTrainer<'bot> {
    pub fn new(red_bot: &'bot mut Bot, yellow_bot: &'bot mut Bot) -> Self {
        Self {
            board: Board::new(),
            turn: Chip::Red,
            moves: 0,
            red_bot,
            yellow_bot,
        }
    }

    fn next_turn(&mut self) {
        self.turn = match self.turn {
            Chip::Red => Chip::Yellow,
            Chip::Yellow => Chip::Red,
        }
    }

    pub fn start(mut self) -> GameResult {
        loop {
            let player = match self.turn {
                Chip::Red => &mut self.red_bot,
                Chip::Yellow => &mut self.yellow_bot,
            };
            let choice = player.choose(self.board);
            let column = choice.column;
            player.remember_played_choice(choice);

            let placed_row = match self.board.place_chip(column, self.turn) {
                Ok(v) => v,
                Err(_) => {
                    unreachable!("our bot is perfect B)");
                }
            };
            self.moves += 1;
            if self.moves > 6 {
                if let Some(winner) = self.board.winner(column, placed_row) {
                    debug_assert!(winner == self.turn);
                    let (winner, loser) = match self.turn {
                        Chip::Red => (&mut self.red_bot, &mut self.yellow_bot),
                        Chip::Yellow => (&mut self.yellow_bot, &mut self.red_bot),
                    };
                    winner.learn_from_played_choices(Action::Reward(2));
                    loser.learn_from_played_choices(Action::Punish(2));
                    return match self.turn {
                        Chip::Red => GameResult::RedWon,
                        Chip::Yellow => GameResult::YellowWon,
                    };
                }
            }
            if self.board.tied() {
                self.red_bot.learn_from_played_choices(Action::Punish(1));
                self.yellow_bot.learn_from_played_choices(Action::Reward(1));
                return GameResult::Tie;
            }
            self.next_turn();
        }
    }
}

pub struct Bot {
    memory: HashMap<Board, Weight>,
    played_choices: [Choice; Board::COLUMN_LEN * Board::ROW_LEN / 2],
    played_choices_len: usize,
    pub exploration: i32,
    seed: usize,
}

pub enum Action {
    Reward(u32),
    Punish(u32),
}

impl Bot {
    pub fn new(exploration: i32) -> Self {
        let played_choices: [Choice; Board::COLUMN_LEN * Board::ROW_LEN / 2] =
            std::array::from_fn(|_| Choice::blank());
        Self {
            memory: HashMap::new(),
            played_choices,
            played_choices_len: 0,
            exploration,
            seed: 0x80085,
        }
    }

    fn lesson_severity_from_turn(base: u32, turn: usize) -> i32 {
        let result = 0.02 * (turn as f64).powi(2) + base as f64;
        result as i32
    }

    pub fn learn_from_played_choices(&mut self, action: Action) {
        for idx in 0..self.played_choices_len {
            let Choice { column, board } = self.played_choices[idx];
            let (weights, swapped) = self.get_or_insert_memory_weights(board);
            let column = if swapped {
                Board::COLUMN_LEN - 1 - column
            } else {
                column
            };
            let weight = &mut weights.0[column];
            match action {
                Action::Reward(base) => *weight += Self::lesson_severity_from_turn(base, idx),
                Action::Punish(base) => *weight -= Self::lesson_severity_from_turn(base, idx),
            };
        }
        self.played_choices_len = 0;
    }

    fn get_or_insert_memory_weights(&mut self, board: Board) -> (&mut Weight, bool) {
        let (key, swapped) = if self.memory.contains_key(&board) {
            (board, false)
        } else if self.memory.contains_key(&board.swap()) {
            (board.swap(), true)
        } else {
            self.memory.insert(board, Weight::blank());
            (board, false)
        };
        (
            self.memory
                .get_mut(&key)
                .expect("we just inserted or verified with contains_key"),
            swapped,
        )
    }

    pub fn remember_played_choice(&mut self, choice: Choice) {
        self.played_choices[self.played_choices_len] = choice;
        self.played_choices_len += 1;
    }

    pub fn choose(&mut self, board: Board) -> Choice {
        self.seed += 1;

        let exploration = self.exploration;
        let (weights, swapped) = self.get_or_insert_memory_weights(board);
        let weights: Box<dyn Iterator<Item = i32>> = if swapped {
            Box::new(weights.0.into_iter().rev())
        } else {
            Box::new(weights.0.into_iter())
        };
        let available_choices = board.available_column_choices();
        let available_choices: Vec<_> = available_choices
            .into_iter()
            .zip(weights)
            .enumerate()
            .filter_map(|(col_idx, (is_available, weight))| {
                if is_available {
                    Some((col_idx, weight))
                } else {
                    None
                }
            })
            .collect();

        let (_, max_weight) = available_choices
            .iter()
            .max_by(|(_, left), (_, right)| left.cmp(right))
            .expect("game is not tied");

        let mut available_choices: Vec<_> = available_choices
            .iter()
            .filter_map(|(idx, weight)| {
                let within_threshold = *weight >= max_weight - exploration;
                if within_threshold {
                    Some(*idx)
                } else {
                    None
                }
            })
            .collect();

        let idx = self.seed % available_choices.len();
        let column = available_choices.swap_remove(idx);

        Choice { board, column }
    }
}
