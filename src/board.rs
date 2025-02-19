use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chip {
    Red,
    Yellow,
}
#[derive(Debug, Clone, Copy, PartialEq)]
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

impl Board {
    const COLUMN_LEN: usize = 7;
    const ROW_LEN: usize = 6;

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

    pub fn tied(&self) -> bool {
        self.columns.count_ones() as usize == Self::COLUMN_LEN * Self::ROW_LEN
    }

    pub fn winner(&self, column: usize, row: usize) -> Option<Chip> {
        if column >= Self::COLUMN_LEN || row >= Self::ROW_LEN {
            return None;
        }

        let directions: [(isize, isize); 3] = [(1, 0), (0, 1), (1, 1)];

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

    pub fn available_choices(&self) -> Vec<usize> {
        (0..Self::COLUMN_LEN)
            .filter(|column| {
                let chips = ((self.columns) >> (Self::ROW_BITS_LEN * column)) as usize;
                let last_chip_in_row_mask = padded_mask(
                    Self::CHIP_BITS_LEN,
                    Self::ROW_BITS_LEN - Self::CHIP_BITS_LEN,
                ) as usize;

                chips & last_chip_in_row_mask == 0
            })
            .collect()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        board::{mask, padded_mask},
        Board, Chip,
    };

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
    fn available_choices() {
        let mut board = Board::new();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(1, Chip::Red).unwrap();
        let _ = board.place_chip(2, Chip::Red).unwrap();
        let _ = board.place_chip(3, Chip::Red).unwrap();
        assert_eq!(board.available_choices().len(), 7);
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap();
        let _ = board.place_chip(0, Chip::Red).unwrap_err();
        let _ = board.place_chip(0, Chip::Red).unwrap_err();
        assert_eq!(board.available_choices().len(), 6);
    }
}
