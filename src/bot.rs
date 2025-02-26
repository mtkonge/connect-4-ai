#![allow(dead_code)]
use std::{collections::HashMap, i16};

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

#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
struct Weight([i16; Board::COLUMN_LEN]);

impl Weight {
    pub fn blank() -> Self {
        Self([0; Board::COLUMN_LEN])
    }

    pub const fn from_weights(list: [i16; Board::COLUMN_LEN]) -> Self {
        Self(list)
    }
}

pub struct BotTrainerGameResult<'bot> {
    red_bot: &'bot mut Bot,
    yellow_bot: &'bot mut Bot,
}

pub struct BotTrainerBoardPosition<'bot> {
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
            self.red_bot.clear_played_choices();
            self.yellow_bot.clear_played_choices();

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

pub struct Game {
    pub board: Board,
    pub turn: Chip,
}

impl Game {
    pub fn new() -> Self {
        Self {
            turn: Chip::Red,
            board: Board::new(),
        }
    }

    pub fn next_turn(&mut self) {
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
                self.bot.clear_played_choices();
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
                self.bot.clear_played_choices();
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

impl<'bot> BotTrainerBoardPosition<'bot> {
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
                let game_result = match game.turn {
                    Chip::Red => GameResult::RedWon,
                    Chip::Yellow => GameResult::YellowWon,
                };
                winner.learn_from_board(Chip::Red, &game_result);
                loser.learn_from_board(Chip::Yellow, &game_result);
                winner.clear_played_choices();
                loser.clear_played_choices();
                break game_result;
            } else if game.board.filled() {
                let game_result = GameResult::Tie;
                self.red_bot.learn_from_board(Chip::Red, &game_result);
                self.yellow_bot.learn_from_board(Chip::Yellow, &game_result);
                self.red_bot.clear_played_choices();
                self.yellow_bot.clear_played_choices();
                break game_result;
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

impl<'bot> BotTrainerGameResult<'bot> {
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
                winner.clear_played_choices();
                loser.clear_played_choices();
                break match game.turn {
                    Chip::Red => GameResult::RedWon,
                    Chip::Yellow => GameResult::YellowWon,
                };
            } else if game.board.filled() {
                self.red_bot.learn_from_played_choices(Action::Punish(1));
                self.yellow_bot.learn_from_played_choices(Action::Reward(1));
                self.red_bot.clear_played_choices();
                self.yellow_bot.clear_played_choices();
                break GameResult::Tie;
            }
            game.next_turn();
        }
    }

    pub fn start_with_iterations(mut self, iterations: usize) {
        for iteration in 1..=iterations {
            if iteration % (iterations / 5) == 0 {
                println!("{}%", (iteration * 100) / iterations);
                println!(
                    "red: {}, yellow: {}",
                    self.red_bot.memory.len(),
                    self.yellow_bot.memory.len()
                );
            }
            self.start_match(Game::new());
            std::mem::swap(self.red_bot, self.yellow_bot);
        }
    }
}

/// https://en.wikipedia.org/wiki/Linear_congruential_generator
#[derive(Clone)]
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

#[derive(Clone)]
pub struct Bot {
    memory: HashMap<Board, Weight>,
    played_choices: [Choice; Board::COLUMN_LEN * Board::ROW_LEN / 2],
    played_choices_len: usize,
    pub exploration: i16,
    rand: Rand,
}

pub enum Action {
    Reward(u32),
    Punish(u32),
}

impl Bot {
    pub fn new(exploration: i16, seed: usize) -> Self {
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

    fn lesson_severity_from_turn(&self, turn: usize) -> i16 {
        let last_turn = self.played_choices_len - 1;
        if turn == last_turn {
            return i16::MAX;
        }
        let result = 0.02 * (turn as f64).powi(2);
        result as i16
    }

    pub fn clear_played_choices(&mut self) {
        self.played_choices_len = 0;
    }

    fn learn_from_board(&mut self, bot_chip: Chip, game_result: &GameResult) {
        for idx in 0..self.played_choices_len {
            let last_turn = self.played_choices_len - 1;
            let Choice { column, board } = self.played_choices[idx];
            let (weights, swapped) = self.get_or_insert_memory_weights(board);
            let column = if swapped {
                Board::COLUMN_LEN - 1 - column
            } else {
                column
            };

            let weight = &mut weights.0[column];
            if idx == last_turn {
                match game_result {
                    GameResult::RedWon => match bot_chip {
                        Chip::Red => *weight = i16::MAX,
                        Chip::Yellow => *weight = i16::MIN,
                    },
                    GameResult::YellowWon => match bot_chip {
                        Chip::Red => *weight = i16::MIN,
                        Chip::Yellow => *weight = i16::MAX,
                    },
                    GameResult::Tie => (),
                }
                continue;
            }
            if let Some(new_weight) = weight.checked_add(board.value_of_board(bot_chip)) {
                *weight = new_weight;
            } else {
                if *weight > 0 {
                    *weight = i16::MAX;
                } else {
                    *weight = i16::MIN;
                }
            }
        }
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
            match action {
                Action::Reward(base) => {
                    if lesson_severity == i16::MAX {
                        *weight = i16::MAX
                    } else {
                        if let Some(new_weight) = weight.checked_add(lesson_severity + base as i16)
                        {
                            *weight = new_weight;
                        } else {
                            *weight = i16::MAX;
                        }
                    }
                }
                Action::Punish(base) => {
                    if lesson_severity == i16::MAX {
                        *weight = i16::MIN
                    } else {
                        if let Some(new_weight) = weight.checked_sub(lesson_severity + base as i16)
                        {
                            *weight = new_weight;
                        } else {
                            *weight = i16::MIN;
                        }
                    }
                }
            };
        }
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
        let weights: Box<dyn Iterator<Item = i16>> = if swapped {
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
                let threshold = max_weight.checked_sub(exploration).unwrap_or(i16::MIN);
                let within_threshold = *weight >= threshold;
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

fn copy_from_to<const SRC_LEN: usize, const DEST_LEN: usize>(
    (src, src_idx): (&[u8; SRC_LEN], &mut usize),
    (dest, dest_idx): (&mut [u8; DEST_LEN], &mut usize),
) {
    loop {
        dest[*dest_idx] = src[*src_idx];
        *src_idx += 1;
        *dest_idx += 1;
        if *dest_idx == DEST_LEN || *src_idx == SRC_LEN {
            break;
        }
    }
}

fn serialize_weights(
    board: &Board,
    weight: &Weight,
) -> [u8; std::mem::size_of::<Board>() + std::mem::size_of::<Weight>()] {
    let mut result = [0; std::mem::size_of::<Board>() + std::mem::size_of::<Weight>()];
    let mut result_idx = 0;
    {
        let (left, right) = board.as_pair();
        let (left, right) = (left.to_le_bytes(), right.to_le_bytes());

        copy_from_to((&left, &mut 0), (&mut result, &mut result_idx));
        copy_from_to((&right, &mut 0), (&mut result, &mut result_idx));
    };

    let weight = weight.0;
    let mut weight_idx = 0;
    loop {
        let bytes = weight[weight_idx].to_le_bytes();
        copy_from_to((&bytes, &mut 0), (&mut result, &mut result_idx));
        weight_idx += 1;
        if weight_idx == weight.len() {
            break;
        }
    }

    result
}

fn deserialize_weights(
    bytes: [u8; std::mem::size_of::<Board>() + std::mem::size_of::<Weight>()],
) -> (Board, Weight) {
    let mut byte_idx = 0;

    let board = {
        let mut left = [0; std::mem::size_of::<u64>()];
        copy_from_to((&bytes, &mut byte_idx), (&mut left, &mut 0));
        let mut right = [0; std::mem::size_of::<u32>()];
        copy_from_to((&bytes, &mut byte_idx), (&mut right, &mut 0));
        Board::from_pair((u64::from_le_bytes(left), u32::from_le_bytes(right)))
    };

    let weight = {
        let mut weight_bytes = [[0; std::mem::size_of::<i16>()]; Board::COLUMN_LEN];
        let mut weight_idx = 0;
        loop {
            copy_from_to(
                (&bytes, &mut byte_idx),
                (&mut weight_bytes[weight_idx], &mut 0),
            );
            weight_idx += 1;
            if weight_idx == weight_bytes.len() {
                break;
            }
        }
        let mut weight_idx = 0;
        let mut weight = [0; Board::COLUMN_LEN];
        loop {
            weight[weight_idx] = i16::from_le_bytes(weight_bytes[weight_idx]);
            weight_idx += 1;
            if weight_idx == weight.len() {
                break;
            }
        }
        Weight::from_weights(weight)
    };

    (board, weight)
}

#[cfg(test)]
mod test {
    use crate::board::Board;

    use super::{deserialize_weights, serialize_weights, Weight};

    #[test]
    fn serde() {
        let board = Board::from_pair((0x5823847547321748, 0x42348245));
        let weights =
            Weight::from_weights([0x2813, 0x2891, 0x3931, 0x3931, 0x5219, 0x4294, 0x2148]);

        let result = deserialize_weights(serialize_weights(&board, &weights));

        assert_eq!((board, weights), result);
    }
}
