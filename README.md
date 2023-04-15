# Sudoku Tester

[![Build Status](https://github.com/cyphersnake/sudoku-tester/actions/workflows/rust.yml/badge.svg)](https://github.com/cyphersnake/sudoku-tester/actions)

A simple Rust crate that checks whether a completed Sudoku of standard size (9x9) is valid. It helps you parse a Sudoku grid from a string, and validate the grid by checking for any duplicates in rows, columns, and boxes.

Dependency-free version specifically for rust-playground [here](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=0d8e3f0af288712dd8a2f01a6cfb7788)!

## Features
- Parses a standard size (9x9) Sudoku grid from a string.
- Validates the Sudoku grid by checking for duplicates in rows, columns, and boxes.
- Returns a list of all validation errors found.
- Uses efficient data structures to save memory.

## Usage
Add the following to your Cargo.toml:

```toml
[dependencies]
sudoku-tester = "0.0.0"
```

Then, use the sudoku-tester crate in your code:

```rust
use sudoku_tester::{Sudoku, ValidationError};

fn main() {
    let sudoku: Sudoku = 
        "534678912\n\
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

    match sudoku.validate() {
        Ok(valid_sudoku) => println!("The Sudoku grid is valid:\n{}", valid_sudoku),
        Err(errors) => {
            println!("The Sudoku grid is invalid. Errors found: {errors:?}");
        }
    }
}
```

