#![allow(dead_code)]
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
    red_bot: &'bot mut Bot,
    yellow_bot: &'bot mut Bot,
}

pub struct MinMaxBotTrainer<'bot> {
    bot: &'bot mut Bot,
    bot_turn: Chip,
}

struct GladiatorGame {
    red_bot: Bot,
    yellow_bot: Bot,
    game: Game,
    statistics: GameStatistics,
}

struct GameStatistics {
    red_wins: usize,
    yellow_wins: usize,
    ties: usize,
}

impl GameStatistics {
    pub fn new() -> Self {
        Self {
            red_wins: 0,
            yellow_wins: 0,
            ties: 0,
        }
    }
}

impl GladiatorGame {
    pub fn new(rand: &mut Rand) -> Self {
        let red_bot = Bot::new(5, rand.next());
        let yellow_bot = Bot::new(5, rand.next());
        let game = Game::new();
        let statistics = GameStatistics::new();
        Self {
            red_bot,
            yellow_bot,
            game,
            statistics,
        }
    }

    pub fn new_from_bots(red_bot: Bot, yellow_bot: Bot) -> Self {
        let game = Game::new();
        let statistics = GameStatistics::new();
        Self {
            red_bot,
            yellow_bot,
            game,
            statistics,
        }
    }

    pub fn evaluate(mut self, iterations: usize) -> Bot {
        for _ in 0..iterations {
            let result = loop {
                let player = match self.game.turn {
                    Chip::Red => &mut self.red_bot,
                    Chip::Yellow => &mut self.yellow_bot,
                };

                let choice = player.choose(self.game.board);
                let column_played = choice.column;
                let row_played = self
                    .game
                    .board
                    .place_chip(column_played, self.game.turn)
                    .expect("we only place based on available positions");
                if self.game.board.winner(column_played, row_played).is_some() {
                    break match self.game.turn {
                        Chip::Red => GameResult::RedWon,
                        Chip::Yellow => GameResult::YellowWon,
                    };
                } else if self.game.board.filled() {
                    break GameResult::Tie;
                }
                self.game.next_turn();
            };

            let (red, yellow) = match result {
                GameResult::RedWon => {
                    self.statistics.red_wins += 1;
                    (Action::Reward(10), Action::Punish(10))
                }
                GameResult::YellowWon => {
                    self.statistics.yellow_wins += 1;
                    (Action::Punish(10), Action::Reward(10))
                }
                GameResult::Tie => {
                    self.statistics.ties += 1;
                    (Action::Punish(1), Action::Reward(1))
                }
            };
            self.red_bot.learn_from_played_choices(red);
            self.yellow_bot.learn_from_played_choices(yellow);

            std::mem::swap(&mut self.red_bot, &mut self.yellow_bot);
            std::mem::swap(
                &mut self.statistics.red_wins,
                &mut self.statistics.yellow_wins,
            );
            self.game = Game::new();
        }
        if self.statistics.red_wins > self.statistics.yellow_wins {
            self.red_bot
        } else {
            self.yellow_bot
        }
    }
}

pub struct GladiatorBotTrainer {
    fights: Vec<GladiatorGame>,
    remainder: Option<Bot>,
}

struct Game {
    board: Board,
    turn: Chip,
}

impl Game {
    fn new() -> Self {
        Self {
            turn: Chip::Red,
            board: Board::new(),
        }
    }

    fn next_turn(&mut self) {
        self.turn = self.turn.opposite()
    }
}

enum GameResult {
    RedWon,
    YellowWon,
    Tie,
}

impl GladiatorBotTrainer {
    pub fn new(arena_size: usize) -> Self {
        let mut rand = Rand::new(0x40523);
        let fights = Vec::from_iter((0..arena_size).map(|_| GladiatorGame::new(&mut rand)));
        Self {
            fights,
            remainder: None,
        }
    }

    pub fn the_one_bot_to_rule_them_all(mut self, iterations: usize) -> Bot {
        loop {
            println!("evaluating {} fights...", self.fights.len());
            let mut games = Vec::with_capacity(self.fights.len() / 2);
            std::mem::swap(&mut games, &mut self.fights);
            let mut winners = games.into_iter().map(|v| v.evaluate(iterations));
            loop {
                let Some(current) = winners.next() else {
                    break;
                };
                let Some(partner) = winners.next() else {
                    self.remainder = Some(current);
                    break;
                };
                self.fights
                    .push(GladiatorGame::new_from_bots(current, partner));
            }
            if self.fights.len() == 0 {
                break self
                    .remainder
                    .expect("there can only be one bot left if arena_size > 0");
            }
        }
    }
}

impl<'bot> MinMaxBotTrainer<'bot> {
    pub fn new(bot: &'bot mut Bot) -> Self {
        Self {
            bot,
            bot_turn: Chip::Red,
        }
    }

    fn start_match(&mut self, mut game: Game) -> GameResult {
        loop {
            let placed_column = if game.turn == self.bot_turn {
                let choice = self.bot.choose(game.board);
                let column = choice.column;
                self.bot.remember_played_choice(choice);

                column
            } else {
                let column = match game.board.minmax(self.bot_turn.opposite(), game.turn) {
                    crate::board::Minmaxxing::Result(_) => unreachable!("board is not filled"),
                    crate::board::Minmaxxing::Position(position, _) => position,
                };
                column
            };

            let placed_row = match game.board.place_chip(placed_column, game.turn) {
                Ok(v) => v,
                Err(_) => {
                    unreachable!("our bot is perfect B)");
                }
            };

            if let Some(winner) = game.board.winner(placed_column, placed_row) {
                let action = if winner == self.bot_turn {
                    Action::Reward(10)
                } else {
                    Action::Punish(10)
                };
                self.bot.learn_from_played_choices(action);
                break match winner {
                    Chip::Red => GameResult::RedWon,
                    Chip::Yellow => GameResult::YellowWon,
                };
            } else if game.board.filled() {
                let action = if self.bot_turn == Chip::Red {
                    Action::Punish(1)
                } else {
                    Action::Reward(1)
                };
                self.bot.learn_from_played_choices(action);
                break GameResult::Tie;
            };
            game.next_turn();
        }
    }

    pub fn start_with_iterations(mut self, iterations: usize) {
        for iteration in 1..=iterations {
            if iteration % (iterations / 10) == 0 {
                println!("{}%", (iteration * 100) / iterations);
            }
            self.start_match(Game::new());
            self.bot_turn = self.bot_turn.opposite();
        }
    }
}

impl<'bot> BotTrainer<'bot> {
    pub fn new(red_bot: &'bot mut Bot, yellow_bot: &'bot mut Bot) -> Self {
        Self {
            red_bot,
            yellow_bot,
        }
    }

    fn start_match(&mut self, mut game: Game) -> GameResult {
        loop {
            let player = match game.turn {
                Chip::Red => &mut self.red_bot,
                Chip::Yellow => &mut self.yellow_bot,
            };
            let choice = player.choose(game.board);
            let column = choice.column;
            player.remember_played_choice(choice);

            let placed_row = match game.board.place_chip(column, game.turn) {
                Ok(v) => v,
                Err(_) => {
                    unreachable!("our bot is perfect B)");
                }
            };
            if let Some(winner) = game.board.winner(column, placed_row) {
                debug_assert!(winner == game.turn);
                let (winner, loser) = match game.turn {
                    Chip::Red => (&mut self.red_bot, &mut self.yellow_bot),
                    Chip::Yellow => (&mut self.yellow_bot, &mut self.red_bot),
                };
                winner.learn_from_played_choices(Action::Reward(10));
                loser.learn_from_played_choices(Action::Punish(10));
                break match game.turn {
                    Chip::Red => GameResult::RedWon,
                    Chip::Yellow => GameResult::YellowWon,
                };
            } else if game.board.filled() {
                self.red_bot.learn_from_played_choices(Action::Punish(1));
                self.yellow_bot.learn_from_played_choices(Action::Reward(1));
                break GameResult::Tie;
            }
            game.next_turn();
        }
    }

    pub fn start_with_iterations(mut self, iterations: usize) {
        for iteration in 1..=iterations {
            if iteration % (iterations / 5) == 0 {
                println!("{}%", (iteration * 100) / iterations);
            }
            self.start_match(Game::new());
            std::mem::swap(self.red_bot, self.yellow_bot);
        }
    }
}

/// https://en.wikipedia.org/wiki/Linear_congruential_generator
struct Rand(usize);

impl Rand {
    pub const MODULUS: usize = 2_usize.pow(31);
    pub const MULTIPLIER: usize = 1103515245;
    pub const INCREMENT: usize = 12345;

    pub const fn new(seed: usize) -> Self {
        Self(seed)
    }

    pub fn next(&mut self) -> usize {
        self.0 = (Self::MULTIPLIER * self.0 + Self::INCREMENT) % Self::MODULUS;
        self.0
    }
}

pub struct Bot {
    memory: HashMap<Board, Weight>,
    played_choices: [Choice; Board::COLUMN_LEN * Board::ROW_LEN / 2],
    played_choices_len: usize,
    pub exploration: i32,
    rand: Rand,
}

pub enum Action {
    Reward(u32),
    Punish(u32),
}

impl Bot {
    pub fn new(exploration: i32, seed: usize) -> Self {
        let played_choices: [Choice; Board::COLUMN_LEN * Board::ROW_LEN / 2] =
            std::array::from_fn(|_| Choice::blank());
        Self {
            memory: HashMap::new(),
            played_choices,
            played_choices_len: 0,
            exploration,
            rand: Rand::new(seed),
        }
    }

    fn lesson_severity_from_turn(&self, turn: usize) -> i32 {
        let last_turn = self.played_choices_len - 1;
        if turn == last_turn {
            return i32::MAX;
        }
        let result = 0.02 * (turn as f64).powi(2);
        result as i32
    }

    pub fn learn_from_played_choices(&mut self, action: Action) {
        for idx in 0..self.played_choices_len {
            let Choice { column, board } = self.played_choices[idx];
            let lesson_severity = self.lesson_severity_from_turn(idx);
            let (weights, swapped) = self.get_or_insert_memory_weights(board);
            let column = if swapped {
                Board::COLUMN_LEN - 1 - column
            } else {
                column
            };
            let weight = &mut weights.0[column];
            if *weight == i32::MAX || *weight == i32::MIN {
                continue;
            }
            match action {
                Action::Reward(base) => {
                    if lesson_severity == i32::MAX {
                        *weight = i32::MAX
                    } else {
                        *weight += lesson_severity + base as i32
                    }
                }
                Action::Punish(base) => {
                    if lesson_severity == i32::MAX {
                        *weight = i32::MIN
                    } else {
                        *weight -= lesson_severity + base as i32
                    }
                }
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
                if *max_weight == i32::MIN {
                    return Some(*idx);
                }
                let within_threshold = *weight >= max_weight - exploration;
                if within_threshold {
                    Some(*idx)
                } else {
                    None
                }
            })
            .collect();

        let idx = self.rand.next() % available_choices.len();
        let column = available_choices.swap_remove(idx);

        Choice { board, column }
    }
}
