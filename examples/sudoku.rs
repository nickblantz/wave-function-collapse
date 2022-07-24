use bitvec::{array::BitArray, order::Lsb0};
use std::{fmt, time::SystemTime};

use wave_function_collapse::{cell::Cell, solver::Solver};

const STATES: usize = 9;
const SIDE_LEN: usize = 9;
const BOARD_SIZE: usize = SIDE_LEN * SIDE_LEN;

type CellStorage = [u16; 1];
type CellState = BitArray<CellStorage, Lsb0>;
type SudokuCell = Cell<CellStorage, STATES>;
type BoardState = [SudokuCell; BOARD_SIZE];

fn adjacencies(i: usize) -> Vec<usize> {
    Box::new(
        [0, 1, 2, 9, 10, 11, 18, 19, 20]
            .iter()
            .map(move |&x| (i / (27)) * (27) + (i % 9 / 3) * 3 + x)
            .chain((0..9).map(move |x| (i / 9) * 9 + x))
            .chain((0..9).map(move |x| (i % 9) + x * 9))
            .filter(move |&x| i != x),
    )
    .collect()
}

fn state_reducer(cell: (usize, &SudokuCell), _: usize) -> CellState {
    cell.1.state()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state =
        parse("6.....5.9.7..4..6.4........51.4...37....63.........9....29.8...........2.9.7.13..")?;
    let mut solver = Solver::new(state, adjacencies, state_reducer);

    let start_time = SystemTime::now();
    solver.solve();
    let elapsed = start_time.elapsed()?;

    print_board(&solver);
    println!("Took {:.4} ms", elapsed.as_secs_f64() * 1000.0);

    Ok(())
}

#[derive(Debug)]
pub enum ParseError {
    InvalidSize(usize),
    InvalidInput(usize, char),
    InternalError,
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::InvalidSize(i) => {
                write!(f, "A board was provided with an invalid length of {}", i)?
            }
            Self::InvalidInput(i, c) => write!(f, "Character {} at position {} is invalid", c, i)?,
            Self::InternalError => write!(f, "An internal error has occurred")?,
        }

        Ok(())
    }
}

fn parse(raw: &str) -> Result<BoardState, ParseError> {
    if raw.len() != BOARD_SIZE {
        return Err(ParseError::InvalidSize(raw.len()));
    }

    let char_map = |c: (usize, char)| -> Result<SudokuCell, ParseError> {
        match c.1 {
            '.' => Ok(Cell::default()),
            '1' => Ok(Cell::from(0)),
            '2' => Ok(Cell::from(1)),
            '3' => Ok(Cell::from(2)),
            '4' => Ok(Cell::from(3)),
            '5' => Ok(Cell::from(4)),
            '6' => Ok(Cell::from(5)),
            '7' => Ok(Cell::from(6)),
            '8' => Ok(Cell::from(7)),
            '9' => Ok(Cell::from(8)),
            _ => Err(ParseError::InvalidInput(c.0, c.1)),
        }
    };

    match TryInto::<[SudokuCell; BOARD_SIZE]>::try_into(
        raw.chars()
            .enumerate()
            .map(char_map)
            .collect::<Result<Vec<SudokuCell>, ParseError>>()?,
    ) {
        Ok(state) => Ok(state),
        Err(_) => Err(ParseError::InternalError),
    }
}

fn print_board(solver: &Solver<CellStorage, STATES, BOARD_SIZE>) {
    let state = solver.state();
    for i in 0..state.len() {
        let cell = &state[i];
        match cell.result() {
            Some(n) => print!("{} ", n + 1),
            None => print!("({}) ", cell.entropy()),
        }

        if (i + 1) % 3 == 0 {
            print!("  ");
        }

        if (i + 1) % 9 == 0 {
            println!();
        }

        if (i + 1) % 27 == 0 {
            println!();
        }
    }
}
