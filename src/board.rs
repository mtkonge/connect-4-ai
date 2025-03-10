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
    column_pair: (u64, u32),
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
    Result(i16),
    Position(usize, i16),
}

impl Board {
    pub const COLUMN_LEN: usize = 7;
    pub const ROW_LEN: usize = 6;

    const ROW_BITS_LEN: usize = Self::ROW_LEN * Self::CHIP_BITS_LEN;
    const CHIP_BITS_LEN: usize = 2;

    pub fn new() -> Self {
        Self {
            column_pair: (0, 0),
        }
    }

    pub fn place_chip(&mut self, column: usize, chip: Chip) -> Result<usize, PlaceChipError> {
        if column >= Self::COLUMN_LEN {
            return Err(PlaceChipError::InvalidColumn);
        }
        let columns = self.as_u128();
        let chips = (columns >> (Self::ROW_BITS_LEN * column)) & mask(Self::ROW_BITS_LEN);
        let chips_placed = chips.count_ones() as usize;
        if chips_placed >= Self::ROW_LEN {
            return Err(PlaceChipError::ColumnOccupied);
        }
        let row = chips_placed;
        self.set_chip_at(column, row, chip);
        Ok(row)
    }

    fn chip_at(&self, column: usize, row: usize) -> Option<Chip> {
        let columns = self.as_u128();
        let chips = (columns >> (Self::ROW_BITS_LEN * column)) as usize;
        let chip = ((chips) >> (Self::CHIP_BITS_LEN * row)) & mask(Self::CHIP_BITS_LEN) as usize;
        match chip {
            0b00 => None,
            0b01 => Some(Chip::Red),
            0b10 => Some(Chip::Yellow),
            _ => unreachable!("invalid bit pattern"),
        }
    }

    pub fn swap(&self) -> Self {
        let columns = self.as_u128();
        let mut swapped_columns = 0u128;
        for column in 0..Self::COLUMN_LEN {
            let row = (columns >> (Board::ROW_BITS_LEN * column)) & mask(Self::ROW_BITS_LEN);
            let rev_position = Self::COLUMN_LEN - 1 - column;
            let rev = row << (Self::ROW_BITS_LEN * rev_position);
            swapped_columns |= rev
        }
        Board::from_pair(Board::pair_from_u128(swapped_columns))
    }

    pub const fn as_pair(&self) -> (u64, u32) {
        self.column_pair
    }

    const fn as_u128(&self) -> u128 {
        (self.column_pair.0 as u128) << std::mem::size_of::<u32>() * 8
            | (self.column_pair.1 as u128)
    }

    pub const fn from_pair(columns: (u64, u32)) -> Self {
        Self {
            column_pair: columns,
        }
    }

    const fn pair_from_u128(value: u128) -> (u64, u32) {
        let v64 = ((value >> std::mem::size_of::<u32>() * 8) & mask(64)) as u64;
        let v32 = (value & mask(32)) as u32;
        (v64, v32)
    }

    pub fn filled(&self) -> bool {
        let ones = self.column_pair.0.count_ones() + self.column_pair.1.count_ones();
        ones as usize == Self::COLUMN_LEN * Self::ROW_LEN
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
        self.column_pair = Self::pair_from_u128(self.as_u128() | (chip << offset));
    }

    pub fn available_column_choices(&self) -> [bool; Self::COLUMN_LEN] {
        std::array::from_fn(|column| {
            let columns = self.as_u128();
            let chips = (columns >> (Self::ROW_BITS_LEN * column)) as usize;
            let last_chip_in_row_mask = padded_mask(
                Self::CHIP_BITS_LEN,
                Self::ROW_BITS_LEN - Self::CHIP_BITS_LEN,
            ) as usize;

            chips & last_chip_in_row_mask == 0
        })
    }

    fn minmax_children(&self, maximizer: Chip, turn: Chip, depth: u8) -> Minmaxxing {
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
                    board.minmax_after_move(maximizer, turn.opposite(), pos, depth),
                )
            })
            .map(|(column, result)| match result {
                Minmaxxing::Position(_, v) => (column, v),
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

    pub fn minmax(&self, maximizer: Chip, turn: Chip) -> Minmaxxing {
        const DEPTH: u8 = 5;

        self.minmax_children(maximizer, turn, DEPTH)
    }

    fn minmax_after_move(&self, maximizer: Chip, turn: Chip, pos: Move, depth: u8) -> Minmaxxing {
        if self.filled() {
            return Minmaxxing::Result(0);
        }
        if let Some(winner) = self.winner(pos.column, pos.row) {
            if maximizer == winner {
                return Minmaxxing::Result(1000);
            } else {
                return Minmaxxing::Result(-1000);
            }
        }

        if depth == 0 {
            let value = self.value_of_board(maximizer);
            return Minmaxxing::Result(value * 8);
        }

        self.minmax_children(maximizer, turn, depth - 1)
    }

    pub fn value_of_board(&self, maximizer: Chip) -> i16 {
        let mut value = 0;
        for col in 0..Self::COLUMN_LEN {
            for row in 0..Self::ROW_LEN {
                match self.win_possibilities_at_position(col, row) {
                    Some((chip, points)) if chip == maximizer => value += points,
                    Some((_chip, points)) => value -= points,
                    None => continue,
                }
            }
        }
        value
    }

    fn win_possibilities_at_position(&self, column: usize, row: usize) -> Option<(Chip, i16)> {
        if column >= Self::COLUMN_LEN || row >= Self::ROW_LEN {
            return None;
        }

        let directions: [(isize, isize); 4] = [(1, -1), (1, 0), (0, 1), (1, 1)];

        let player = self.chip_at(column, row)?;

        let possible_wins = directions
            .iter()
            .map(|(column_dir, row_dir)| {
                (0..=3)
                    .map(|idx| (idx - 3..=idx))
                    .map(|mut stripe| {
                        stripe.all(|idx| {
                            let (column, row) = (
                                column as isize + column_dir * idx,
                                row as isize + row_dir * idx,
                            );

                            if !(0..Self::COLUMN_LEN as isize).contains(&column)
                                || !(0..Self::ROW_LEN as isize).contains(&row)
                            {
                                return false;
                            }

                            let (column, row) = (column as usize, row as usize);
                            match self.chip_at(column, row) {
                                Some(other) if player == other => true,
                                None => true,
                                Some(_) => false,
                            }
                        })
                    })
                    .filter(|&available| available)
                    .count() as i16
            })
            .sum();

        Some((player, possible_wins))
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
                        Some(Chip::Red) => "\x1b[0;31m0\x1b[0m",
                        Some(Chip::Yellow) => "\x1b[0;33m0\x1b[0m",
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
    fn from_pair_to_pair_ok() {
        let mut board = Board::new();
        board.set_chip_at(4, 3, Chip::Red);
        board.set_chip_at(6, 0, Chip::Red);
        assert_eq!(Board::from_pair(board.as_pair()), board)
    }

    #[test]
    fn set_get_last_column() {
        let mut board = Board::new();
        board.set_chip_at(0, 0, Chip::Red);
        board.set_chip_at(6, 0, Chip::Red);
        board.set_chip_at(6, 1, Chip::Red);
        assert_eq!(board.chip_at(6, 0), Some(Chip::Red));
    }

    #[test]
    fn swap() {
        let mut board = Board::new();
        board.set_chip_at(2, 2, Chip::Red);
        board.set_chip_at(3, 4, Chip::Yellow);
        println!("{board}");
        let board = board.swap();
        println!("{board}");
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

    #[test]
    fn win_possibilities_at_position() {
        // (&self, column: usize, row: usize)
        let mut board = Board::new();

        let _ = board.place_chip(0, Chip::Yellow).unwrap();
        let _ = board.place_chip(1, Chip::Yellow).unwrap();
        let _ = board.place_chip(2, Chip::Yellow).unwrap();
        assert_eq!(
            board.win_possibilities_at_position(2, 0),
            Some((Chip::Yellow, 5))
        )
    }

    #[test]
    fn win_possibilities_at_position_for_document() {
        let mut board = Board::new();

        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();

        let _ = board.place_chip(2, Chip::Yellow).unwrap();

        let _ = board.place_chip(3, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Yellow).unwrap();

        let _ = board.place_chip(5, Chip::Yellow).unwrap();
        let _ = board.place_chip(5, Chip::Yellow).unwrap();

        let _ = board.place_chip(6, Chip::Red).unwrap();
        let _ = board.place_chip(6, Chip::Yellow).unwrap();

        println!("{board}");

        assert_eq!(
            board.win_possibilities_at_position(3, 1),
            Some((Chip::Yellow, 8))
        )
    }
}
