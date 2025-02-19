use std::{
    fmt::Display,
    io::{self, Write},
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Chip {
    Red,
    Yellow,
}
#[derive(Debug, PartialEq, Clone)]
struct Board {
    columns: [[Option<Chip>; Self::ROW_LEN]; Self::COLUMN_LEN],
}

enum PlaceChipError {
    SpaceOccupied,
    InvalidColumn,
}

impl Board {
    const COLUMN_LEN: usize = 7;
    const ROW_LEN: usize = 6;

    fn new() -> Self {
        Self {
            columns: [[None; Self::ROW_LEN]; Self::COLUMN_LEN],
        }
    }

    fn place_chip(&mut self, column: usize, chip: Chip) -> Result<usize, PlaceChipError> {
        if column >= Self::COLUMN_LEN {
            return Err(PlaceChipError::InvalidColumn);
        }
        let empty_row = self.columns[column]
            .iter_mut()
            .enumerate()
            .find(|(_, chip)| chip.is_none());
        let Some((row_idx, row)) = empty_row else {
            return Err(PlaceChipError::SpaceOccupied);
        };
        *row = Some(chip);
        Ok(row_idx)
    }

    fn from_bits(columns: u128) -> Self {
        const CHIP_BITS_LENGTH: usize = 2;
        let columns = std::array::from_fn(|column_index| {
            let column_offset = Self::ROW_LEN * CHIP_BITS_LENGTH * column_index;
            let row = columns >> column_offset;
            std::array::from_fn(|item_index| {
                let item_offset = CHIP_BITS_LENGTH * item_index;
                let item_mask = 0b11;
                let item = (row >> item_offset) & item_mask;
                match item {
                    0b00 => None,
                    0b10 => Some(Chip::Red),
                    0b01 => Some(Chip::Yellow),
                    _ => unreachable!("result is masked with 0b11"),
                }
            })
        });

        Self { columns }
    }

    fn as_bits(&self) -> u128 {
        self.columns
            .iter()
            .rev()
            .map(|row| {
                let row = row
                    .iter()
                    .rev()
                    .map(|item| match item {
                        Some(Chip::Red) => 0b10,
                        Some(Chip::Yellow) => 0b01,
                        None => 0b00,
                    })
                    .fold(0u128, |row, item| (row << 2) | item);
                row
            })
            .fold(0u128, |board, row| (board << (Self::ROW_LEN * 2)) | row)
    }

    fn available_choices(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for (col_idx, col) in self.columns.iter().enumerate() {
            if col[Self::ROW_LEN - 1].is_some() {
                result.push(col_idx);
            }
        }
        result
    }

    fn winner(&self, column: usize, row: usize) -> Option<Chip> {
        if column >= Self::COLUMN_LEN || row >= Self::ROW_LEN {
            return None;
        }

        let directions: [(isize, isize); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        let player = self.columns[column][row]?;

        let is_winner = directions.iter().any(|(column_dir, row_dir)| {
            (1..=3).all(|ext| {
                let (column, row) = (
                    column as isize + column_dir * ext,
                    row as isize + row_dir * ext,
                );
                if !(0..Self::COLUMN_LEN as isize).contains(&column)
                    || !(0..Self::ROW_LEN as isize).contains(&row)
                {
                    return false;
                }

                let (column, row) = (column as usize, row as usize);
                self.columns[column][row].is_some_and(|v| v == player)
            })
        });

        if is_winner {
            Some(player)
        } else {
            None
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let column_indicators =
            (0..self.columns.len()).fold(" ".to_string(), |acc, v| format!("{acc}{v} "));
        let row_top = (0..self.columns.len()).fold("+".to_string(), |acc, _| format!("{acc}-+"));
        let mut result = column_indicators + "\n" + &row_top + "\n";
        for row in (0..Self::ROW_LEN).rev() {
            let mut row_inputs = Vec::new();
            for col in 0..Self::COLUMN_LEN {
                match self.columns[col][row] {
                    Some(Chip::Red) => row_inputs.push("\x1b[0;31mO\x1b[0m"),
                    Some(Chip::Yellow) => row_inputs.push("\x1b[0;33mO\x1b[0m"),
                    None => row_inputs.push(" "),
                }
            }
            result += &row_inputs
                .iter()
                .fold("|".to_string(), |acc, v| acc + v + "|");
            if row > 0 {
                result += "\n"
            }
        }
        write!(f, "{result}")
    }
}

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
                        PlaceChipError::SpaceOccupied => {
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
            if self.moves == Board::ROW_LEN * Board::COLUMN_LEN {
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
                board: board.clone(),
                column: column_idx,
            };
        }
        return todo!();
    }
}

fn main() {
    loop {
        let game = Game::new();
        game.start();
    }
}
