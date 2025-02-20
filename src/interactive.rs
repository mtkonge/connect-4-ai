#![allow(dead_code)]
use std::io::{self, Write};

use crate::board::{Board, Chip, PlaceChipError};

pub struct InteractiveGame {
    board: Board,
    turn: Chip,
    moves: usize,
}

impl InteractiveGame {
    pub fn new() -> Self {
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

    pub fn start(mut self) {
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
