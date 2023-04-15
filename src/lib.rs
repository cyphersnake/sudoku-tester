#![doc = include_str!("../README.md")]
#![feature(iterator_try_collect)]
use std::{fmt, str::FromStr};

use some_to_err::ErrOr;
use tinyvec::ArrayVec;

const SUDOKU_SIZE: usize = 9;
#[derive(PartialEq, Eq, Debug)]
pub struct Sudoku {
    // NOTE There are Sudoku's that are not standard
    // size, however, I think for simplicity they can
    // be omitted.
    grid: [[u8; SUDOKU_SIZE]; SUDOKU_SIZE],
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.grid.iter().try_for_each(|row| {
            row.iter().enumerate().try_for_each(|(col, val)| {
                if col > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", val)
            })?;

            writeln!(f)
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    WrongSymbol(char),
    WrongRowSize { index: usize, len: usize },
    WrongColumnSize { column_count: usize },
}

impl FromStr for Sudoku {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        use ParseError::*;

        let grid: [[u8; SUDOKU_SIZE]; SUDOKU_SIZE] = s
            .lines()
            .enumerate()
            .map(|(row, line)| {
                line.chars()
                    .map(|c| match c.to_digit(10) {
                        Some(val) => Ok(val as u8),
                        None => Err(WrongSymbol(c)),
                    })
                    .try_collect::<Vec<u8>>()?
                    .try_into()
                    .map_err(|err: Vec<u8>| WrongRowSize {
                        index: row,
                        len: err.len(),
                    })
            })
            .try_collect::<Vec<_>>()?
            .try_into()
            .map_err(|err: Vec<[u8; SUDOKU_SIZE]>| WrongColumnSize {
                column_count: err.len(),
            })?;

        Ok(Sudoku { grid })
    }
}

// Since this vec cannot be greater than 9,
// we can use a data type that takes this into account!
pub type Indexes = ArrayVec<[(usize, usize); 9]>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValidationErrorType {
    Column(usize),
    Row(usize),
    Box(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    Dublication {
        type_: ValidationErrorType,
        value: u8,
        indexes: Indexes,
    },
}

impl Sudoku {
    pub fn validate(self) -> Result<Self, Vec<ValidationError>> {
        #[derive(Default)]
        enum Number {
            #[default]
            Unknown,
            Present {
                row: usize,
                col: usize,
            },
            Corrupted(Indexes),
        }

        impl Number {
            fn indicate(&mut self, new_row: usize, new_col: usize) {
                match self {
                    Self::Unknown => {
                        *self = Self::Present {
                            row: new_row,
                            col: new_col,
                        };
                    }
                    Self::Present { row, col } => {
                        *self = Self::Corrupted(
                            [(*row, *col), (new_row, new_col)].into_iter().collect(),
                        )
                    }
                    Self::Corrupted(ref mut indexes) => {
                        indexes.push((new_row, new_col));
                    }
                };
            }
            fn into_err(self, value: u8, type_: ValidationErrorType) -> Option<ValidationError> {
                match self {
                    Self::Corrupted(indexes) => Some(ValidationError::Dublication {
                        type_,
                        value,
                        indexes,
                    }),
                    _ => None,
                }
            }
        }
        let mut row_seen: [[Number; SUDOKU_SIZE]; SUDOKU_SIZE] = Default::default();
        let mut column_seen: [[Number; SUDOKU_SIZE]; SUDOKU_SIZE] = Default::default();
        let mut boxes_seen: [[Number; SUDOKU_SIZE]; SUDOKU_SIZE] = Default::default();

        for (i, row) in self.grid.iter().enumerate() {
            for (j, val) in row.iter().enumerate() {
                let box_index = (i / 3) * 3 + (j / 3);
                let val = *val as usize;

                row_seen[i][val - 1].indicate(i, j);
                column_seen[j][val - 1].indicate(i, j);
                boxes_seen[box_index][val - 1].indicate(i, j);
            }
        }

        macro_rules! get_validation_errors {
            ($seen:expr, $err_type:ident) => {
                $seen.into_iter().enumerate().flat_map(|(i, values)| {
                    values
                        .into_iter()
                        .enumerate()
                        .filter_map(move |(value, v)| {
                            v.into_err(value as u8 + 1, ValidationErrorType::$err_type(i))
                        })
                })
            };
        }

        // NOTE: A sensible compromise, bypass the elements twice (for a fixed number of variables
        // this does not significantly affect performance), but get all possible errors.
        //
        // If this solution is too sub-optimal, I can replace `indicate(i, j)` with `validate(i, j)?`
        // above and stop after first error
        get_validation_errors!(row_seen, Row)
            .chain(get_validation_errors!(column_seen, Column))
            .chain(get_validation_errors!(boxes_seen, Box))
            .map(Some)
            .collect::<Option<_>>()
            .err_or(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let sudoku: Sudoku = "534678912\n\
             672195348\n\
             198342567\n\
             859761423\n\
             426853791\n\
             713924856\n\
             961537284\n\
             287419635\n\
             345286177"
            .parse()
            .unwrap();
        assert_eq!(
            sudoku,
            Sudoku {
                grid: [
                    [5, 3, 4, 6, 7, 8, 9, 1, 2],
                    [6, 7, 2, 1, 9, 5, 3, 4, 8],
                    [1, 9, 8, 3, 4, 2, 5, 6, 7],
                    [8, 5, 9, 7, 6, 1, 4, 2, 3],
                    [4, 2, 6, 8, 5, 3, 7, 9, 1],
                    [7, 1, 3, 9, 2, 4, 8, 5, 6],
                    [9, 6, 1, 5, 3, 7, 2, 8, 4],
                    [2, 8, 7, 4, 1, 9, 6, 3, 5],
                    [3, 4, 5, 2, 8, 6, 1, 7, 7]
                ]
            }
        );
    }

    #[test]
    fn test_validate_sudoku() {
        let sudoku: Sudoku = "534678912\n\
             672195348\n\
             198342567\n\
             859761423\n\
             426853791\n\
             713924856\n\
             961537284\n\
             287419635\n\
             345286177"
            .parse()
            .unwrap();
        println!("{}", sudoku);

        assert_eq!(
            sudoku.validate(),
            Err(vec![
                ValidationError::Dublication {
                    type_: ValidationErrorType::Row(8),
                    value: 7,
                    indexes: [(8, 7), (8, 8)].into_iter().collect(),
                },
                ValidationError::Dublication {
                    type_: ValidationErrorType::Column(8),
                    value: 7,
                    indexes: [(2, 8), (8, 8)].into_iter().collect(),
                },
                ValidationError::Dublication {
                    type_: ValidationErrorType::Box(8),
                    value: 7,
                    indexes: [(8, 7), (8, 8)].into_iter().collect()
                }
            ])
        );
    }

    #[test]
    fn test_parse_wrong_sudoku_col() {
        let sudoku = "111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111"
            .parse::<Sudoku>()
            .err()
            .unwrap();

        assert_eq!(sudoku, ParseError::WrongColumnSize { column_count: 10 });
    }

    #[test]
    fn test_parse_wrong_sudoku_row() {
        let sudoku = "1111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111"
            .parse::<Sudoku>()
            .err()
            .unwrap();

        assert_eq!(sudoku, ParseError::WrongRowSize { index: 0, len: 10 });
    }

    #[test]
    fn test_parse_wrong_sudoku_symbol() {
        let sudoku = "a11111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111"
            .parse::<Sudoku>()
            .err()
            .unwrap();

        assert_eq!(sudoku, ParseError::WrongSymbol('a'));
    }

    #[test]
    fn test_wrong_sudoku() {
        let sudoku: Sudoku = "111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111\n\
             111111111"
            .parse()
            .unwrap();
        println!("{}", sudoku);

        assert_eq!(sudoku.validate().err().unwrap().len(), 27);
    }
}
