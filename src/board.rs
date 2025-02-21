use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chip {
    Red,
    Yellow,
}

impl Chip {
    pub const fn opposite(&self) -> Self {
        match self {
            Chip::Red => Chip::Yellow,
            Chip::Yellow => Chip::Red,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, Clone, Copy, PartialEq, Hash)]
pub struct Board {
    columns: u128,
}

#[derive(Debug)]
pub enum PlaceChipError {
    ColumnOccupied,
    InvalidColumn,
}

const fn padded_mask(count: usize, padding: usize) -> u128 {
    let mut i = 0;
    let mut result = 0;
    loop {
        if i == count {
            break result << padding;
        }
        result = (result << 1) | 0b1;
        i += 1;
    }
}

const fn mask(count: usize) -> u128 {
    let mut i = 0;
    let mut result = 0;
    loop {
        if i == count {
            break result;
        }
        result = (result << 1) | 0b1;
        i += 1;
    }
}

struct Move {
    column: usize,
    row: usize,
}

pub enum Minmaxxing {
    Result(i8),
    Position(usize, i8),
}

impl Board {
    pub const COLUMN_LEN: usize = 7;
    pub const ROW_LEN: usize = 6;

    const ROW_BITS_LEN: usize = Self::ROW_LEN * Self::CHIP_BITS_LEN;
    const CHIP_BITS_LEN: usize = 2;

    pub fn new() -> Self {
        Self { columns: 0 }
    }

    pub fn place_chip(&mut self, column: usize, chip: Chip) -> Result<usize, PlaceChipError> {
        if column >= Self::COLUMN_LEN {
            return Err(PlaceChipError::InvalidColumn);
        }
        let chips = ((self.columns) >> (Self::ROW_BITS_LEN * column)) & mask(Self::ROW_BITS_LEN);
        let chips_placed = chips.count_ones() as usize;
        if chips_placed >= Self::ROW_LEN {
            return Err(PlaceChipError::ColumnOccupied);
        }
        let row = chips_placed;
        self.set_chip_at(column, row, chip);
        Ok(row)
    }

    fn chip_at(&self, column: usize, row: usize) -> Option<Chip> {
        let chips = ((self.columns) >> (Self::ROW_BITS_LEN * column)) as usize;
        let chip = ((chips) >> (Self::CHIP_BITS_LEN * row)) & mask(Self::CHIP_BITS_LEN) as usize;
        match chip {
            0b00 => None,
            0b01 => Some(Chip::Red),
            0b10 => Some(Chip::Yellow),
            _ => unreachable!("invalid bit pattern"),
        }
    }

    pub fn swap(&self) -> Self {
        let mut result = Board::new();
        for column in 0..Self::COLUMN_LEN {
            let row = (self.columns >> (Board::ROW_BITS_LEN * column)) & mask(Self::ROW_BITS_LEN);
            let rev = Self::COLUMN_LEN - 1 - column;
            result.columns |= row << (Self::ROW_BITS_LEN * rev);
        }
        result
    }

    pub fn board_filled(&self) -> bool {
        self.columns.count_ones() as usize == Self::COLUMN_LEN * Self::ROW_LEN
    }

    pub fn winner(&self, column: usize, row: usize) -> Option<Chip> {
        if column >= Self::COLUMN_LEN || row >= Self::ROW_LEN {
            return None;
        }

        let directions: [(isize, isize); 4] = [(1, -1), (1, 0), (0, 1), (1, 1)];

        let player = self.chip_at(column, row)?;

        let is_winner = directions.iter().any(|(column_dir, row_dir)| {
            (0..=3).any(|min| {
                (min - 3..=min).all(|max| {
                    let (column, row) = (
                        column as isize + column_dir * max,
                        row as isize + row_dir * max,
                    );
                    if !(0..Self::COLUMN_LEN as isize).contains(&column)
                        || !(0..Self::ROW_LEN as isize).contains(&row)
                    {
                        return false;
                    }

                    let (column, row) = (column as usize, row as usize);
                    self.chip_at(column, row).is_some_and(|v| v == player)
                })
            })
        });

        if is_winner {
            Some(player)
        } else {
            None
        }
    }

    fn set_chip_at(&mut self, column: usize, row: usize, chip: Chip) {
        let offset = (Self::ROW_BITS_LEN * column) + (Self::CHIP_BITS_LEN * row);
        let chip = match chip {
            Chip::Red => 0b01,
            Chip::Yellow => 0b10,
        };
        self.columns |= chip << offset;
    }

    pub fn available_column_choices(&self) -> [bool; Self::COLUMN_LEN] {
        std::array::from_fn(|column| {
            let chips = (self.columns >> (Self::ROW_BITS_LEN * column)) as usize;
            let last_chip_in_row_mask = padded_mask(
                Self::CHIP_BITS_LEN,
                Self::ROW_BITS_LEN - Self::CHIP_BITS_LEN,
            ) as usize;

            chips & last_chip_in_row_mask == 0
        })
    }

    pub fn minmax(&self, maximizer: Chip, turn: Chip) -> Minmaxxing {
        const DEPTH: u8 = 4;

        let children = self
            .available_column_choices()
            .into_iter()
            .enumerate()
            .filter_map(|(column, available)| if available { Some(column) } else { None })
            .map(|column| {
                let mut board = self.clone();
                let row = board
                    .place_chip(column, turn)
                    .expect("making move based on available choices");
                (Move { column, row }, board)
            })
            .map(|(pos, board)| {
                (
                    pos.column,
                    board.minmax_inner(maximizer, turn.opposite(), pos, DEPTH),
                )
            })
            .map(|(column, result)| match result {
                Minmaxxing::Position(_, v) => (column, v.clamp(-1, 1)),
                Minmaxxing::Result(v) => (column, v),
            });

        let chosen = if turn == maximizer {
            children.max_by(|(_, left_score), (_, right_score)| left_score.cmp(&right_score))
        } else {
            children.min_by(|(_, left_score), (_, right_score)| left_score.cmp(&right_score))
        };

        chosen
            .map(|(column, score)| Minmaxxing::Position(column, score))
            .expect("game is not over")
    }

    fn minmax_inner(&self, maximizer: Chip, turn: Chip, pos: Move, depth: u8) -> Minmaxxing {
        if depth == 0 {
            // TODO: calculate value of board (-100 if enemy can win, 100 if i can win, otherwise,
            // count the # + length of stripes on either side
            // and do some fancy math
            return Minmaxxing::Result(0);
        }
        if self.board_filled() {
            return Minmaxxing::Result(0);
        }
        if let Some(winner) = self.winner(pos.column, pos.row) {
            if maximizer == winner {
                return Minmaxxing::Result(2);
            } else {
                return Minmaxxing::Result(-2);
            }
        }

        let children = self
            .available_column_choices()
            .into_iter()
            .enumerate()
            .filter_map(|(column, available)| if available { Some(column) } else { None })
            .map(|column| {
                let mut board = self.clone();
                let row = board
                    .place_chip(column, turn)
                    .expect("making move based on available choices");
                (Move { column, row }, board)
            })
            .map(|(pos, board)| {
                (
                    pos.column,
                    board.minmax_inner(maximizer, turn.opposite(), pos, depth - 1),
                )
            })
            .map(|(column, result)| match result {
                Minmaxxing::Position(_, v) => (column, v.clamp(-1, 1)),
                Minmaxxing::Result(v) => (column, v),
            });

        let chosen = if turn == maximizer {
            children.max_by(|(_, left_score), (_, right_score)| left_score.cmp(&right_score))
        } else {
            children.min_by(|(_, left_score), (_, right_score)| left_score.cmp(&right_score))
        };

        chosen
            .map(|(column, score)| Minmaxxing::Position(column, score))
            .expect("game is not over")
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let column_indicators = {
            let column_indicators: [_; Self::COLUMN_LEN] =
                std::array::from_fn(|column| column.to_string());
            column_indicators.join(" ")
        };
        let rows = {
            let mut rows: [_; Self::ROW_LEN] = std::array::from_fn(|row| {
                let columns: [_; Self::COLUMN_LEN] =
                    std::array::from_fn(|column| match self.chip_at(column, row) {
                        Some(Chip::Red) => "\x1b[0;31mO\x1b[0m",
                        Some(Chip::Yellow) => "\x1b[0;33mO\x1b[0m",
                        None => " ",
                    });
                format!("|{}|", columns.join("|"))
            });
            rows.reverse();
            rows.join("\n")
        };
        write!(f, " {column_indicators} \n{rows}")
    }
}

#[cfg(test)]
mod test {
    use crate::board::{mask, padded_mask, Board, Chip};

    #[test]
    fn test_mask() {
        assert_eq!(mask(0), 0b0);
        assert_eq!(mask(1), 0b1);
        assert_eq!(mask(5), 0b11111);
        assert_eq!(mask(12), 0b11_11_11_11_11_11);

        assert_eq!(padded_mask(0, 0), 0b0);
        assert_eq!(padded_mask(1, 0), 0b1);
        assert_eq!(padded_mask(5, 5), 0b11111_00000);
        assert_eq!(padded_mask(12, 3), 0b11_11_11_11_11_11_000);
    }

    #[test]
    fn set_get() {
        let mut board = Board::new();
        board.set_chip_at(2, 2, Chip::Red);
        board.set_chip_at(3, 4, Chip::Yellow);
        assert_eq!(board.chip_at(2, 2), Some(Chip::Red));
        assert_eq!(board.chip_at(3, 4), Some(Chip::Yellow));
        assert_eq!(board.chip_at(4, 3), None);
    }

    #[test]
    fn swap() {
        let mut board = Board::new();
        board.set_chip_at(2, 2, Chip::Red);
        board.set_chip_at(3, 4, Chip::Yellow);
        let board = board.swap();
        let column_end_position = Board::COLUMN_LEN - 1;
        assert_eq!(board.chip_at(column_end_position - 2, 2), Some(Chip::Red));
        assert_eq!(
            board.chip_at(column_end_position - 3, 4),
            Some(Chip::Yellow)
        );
    }

    #[test]
    fn place() {
        let mut board = Board::new();
        let _ = board.place_chip(2, Chip::Red);
        let _ = board.place_chip(3, Chip::Yellow);
        let _ = board.place_chip(3, Chip::Red);
        assert_eq!(board.chip_at(2, 0), Some(Chip::Red));
        assert_eq!(board.chip_at(3, 0), Some(Chip::Yellow));
        assert_eq!(board.chip_at(3, 1), Some(Chip::Red));
    }

    #[test]
    fn winner() {
        let mut board = Board::new();
        let _ = board.place_chip(0, Chip::Red);
        let _ = board.place_chip(1, Chip::Red);
        let _ = board.place_chip(2, Chip::Red);
        let _ = board.place_chip(3, Chip::Red);
        assert_eq!(board.winner(3, 0), Some(Chip::Red));
        assert_eq!(board.winner(0, 0), Some(Chip::Red));
        assert_eq!(board.winner(2, 0), Some(Chip::Red));
        assert_eq!(board.winner(2, 1), None);
        assert_eq!(board.winner(4, 0), None);
    }

    #[test]
    fn swap_boawd() {
        let mut board = Board::new();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(1, Chip::Red).unwrap();
        let _ = board.place_chip(2, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Red).unwrap();
        assert!(board.available_column_choices().into_iter().all(|v| v));
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap_err();
        let _ = board.place_chip(0, Chip::Red).unwrap_err();
        assert_eq!(
            board
                .available_column_choices()
                .into_iter()
                .filter(|&v| v)
                .count(),
            6
        );
    }

    #[test]
    fn can_win_diagonally_ltr() {
        let mut board = Board::new();

        let _ = board.place_chip(0, Chip::Yellow).unwrap();

        let _ = board.place_chip(1, Chip::Red).unwrap();
        let _ = board.place_chip(1, Chip::Yellow).unwrap();

        let _ = board.place_chip(2, Chip::Red).unwrap();
        let _ = board.place_chip(2, Chip::Red).unwrap();
        let _ = board.place_chip(2, Chip::Yellow).unwrap();

        let _ = board.place_chip(3, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Yellow).unwrap();

        assert_eq!(board.winner(0, 0), Some(Chip::Yellow));
        assert_eq!(board.winner(1, 1), Some(Chip::Yellow));
        assert_eq!(board.winner(2, 2), Some(Chip::Yellow));
        assert_eq!(board.winner(3, 3), Some(Chip::Yellow));
    }

    #[test]
    fn can_win_diagonally_rtl() {
        let mut board = Board::new();

        let _ = board.place_chip(3, Chip::Yellow).unwrap();

        let _ = board.place_chip(2, Chip::Red).unwrap();
        let _ = board.place_chip(2, Chip::Yellow).unwrap();

        let _ = board.place_chip(1, Chip::Red).unwrap();
        let _ = board.place_chip(1, Chip::Red).unwrap();
        let _ = board.place_chip(1, Chip::Yellow).unwrap();

        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Yellow).unwrap();

        assert_eq!(board.winner(3, 0), Some(Chip::Yellow));
        assert_eq!(board.winner(2, 1), Some(Chip::Yellow));
        assert_eq!(board.winner(1, 2), Some(Chip::Yellow));
        assert_eq!(board.winner(0, 3), Some(Chip::Yellow));
    }

    #[test]
    fn available_choices() {
        let mut board = Board::new();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(1, Chip::Red).unwrap();
        let _ = board.place_chip(2, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Red).unwrap();
        assert!(board.available_column_choices().into_iter().all(|v| v));
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap_err();
        let _ = board.place_chip(0, Chip::Red).unwrap_err();
        assert_eq!(
            board
                .available_column_choices()
                .into_iter()
                .filter(|&v| v)
                .count(),
            6
        );
    }
}
