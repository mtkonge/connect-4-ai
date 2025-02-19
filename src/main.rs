use std::io::{self, Write};

use board::{Board, Chip};

use crate::board::PlaceChipError;

mod board;

struct Game {
    board: Board,
    turn: Chip,
    moves: usize,
}

impl Game {
    fn new() -> Self {
        Self {
            board: Board::new(),
            turn: Chip::Red,
            moves: 0,
        }
    }

    fn next_turn(&mut self) {
        self.turn = match self.turn {
            Chip::Red => Chip::Yellow,
            Chip::Yellow => Chip::Red,
        }
    }

    fn start(mut self) {
        println!("{}", self.board);

        loop {
            println!();
            print!("Which column would you like to place your chip? (0-6) % ");
            io::stdout()
                .lock()
                .flush()
                .expect("should be able to flush stdout");
            let mut column = String::new();
            io::stdin()
                .read_line(&mut column)
                .expect("should be able to read line from stdin");
            println!();

            let column = column.trim();
            let Ok(column) = column.parse() else {
                println!("Invalid column '{column}'");
                continue;
            };
            let placed_row = match self.board.place_chip(column, self.turn) {
                Ok(v) => v,
                Err(err) => {
                    let msg = match err {
                        PlaceChipError::ColumnOccupied => {
                            format!("Column '{column}' is full, pick another column")
                        }

                        PlaceChipError::InvalidColumn => format!("Invalid column '{column}'"),
                    };
                    println!("{msg}");
                    continue;
                }
            };
            self.next_turn();
            self.moves += 1;
            println!("{}", self.board);
            if self.moves > 6 {
                if let Some(winner) = self.board.winner(column, placed_row) {
                    println!("{:?}", winner);
                    break;
                }
            }
            if self.board.tied() {
                println!("tie");
                break;
            }
        }
    }
}

#[derive(PartialEq, Clone)]
struct Choice {
    board: Board,
    column: usize,
}

struct WeightedChoice {
    choice: Choice,
    weight: i32,
}

struct Player {
    memory: Vec<WeightedChoice>,
    current_choices: Vec<Choice>,
    choice_weight_threshold: usize,
    chip: Chip,
}

enum Action {
    Reward(u32),
    Punish(u32),
}

impl Player {
    fn new(choice_weight_threshold: usize, chip: Chip) -> Self {
        Self {
            memory: Vec::new(),
            current_choices: Vec::new(),
            choice_weight_threshold,
            chip,
        }
    }

    fn apply_action_to_current_choices(&mut self, action: Action) {
        for choice in self.current_choices.iter_mut() {
            let found_choice = self
                .memory
                .iter_mut()
                .find(|weighted_choice| weighted_choice.choice == *choice);

            if let Some(choice) = found_choice {
                match action {
                    Action::Reward(amount) => choice.weight += amount as i32,
                    Action::Punish(amount) => choice.weight -= amount as i32,
                };
            } else {
                self.memory.push(WeightedChoice {
                    choice: choice.clone(),
                    weight: match action {
                        Action::Reward(amount) => amount as i32,
                        Action::Punish(amount) => -(amount as i32),
                    },
                });
            }
        }
    }

    fn clear_current_choices(&mut self) {
        self.current_choices = Vec::new()
    }

    fn best_choice(&self) -> Option<&WeightedChoice> {
        self.memory
            .iter()
            .max_by(|left, right| left.weight.cmp(&right.weight))
    }

    fn choose_column(&self, board: Board) -> usize {
        let choice_results: Vec<Board> = Vec::new();
        for column_idx in board.available_choices() {
            let choice = Choice {
                board,
                column: column_idx,
            };
        }
        todo!()
    }
}

fn main() {
    loop {
        let game = Game::new();
        game.start();
    }
}
