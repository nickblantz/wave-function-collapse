use bitvec::{array::BitArray, order::Lsb0};
use std::{collections::HashSet, fmt, time::SystemTime};

use wave_function_collapse::{
    cell::Cell,
    solver::{Solver, SolverBuilder},
};

const STATES: usize = 9;
const ROW_LEN: usize = 9;
const COL_LEN: usize = 9;
const BOARD_SIZE: usize = ROW_LEN * COL_LEN;

type CellStorage = u16;
type CellState = BitArray<CellStorage, Lsb0>;
type SudokuCell = Cell<CellStorage, STATES>;
type BoardState = [SudokuCell; BOARD_SIZE];

fn neighbors(i: usize) -> Vec<usize> {
    const COL_LEN_F: f64 = COL_LEN as f64;
    let y = i / ROW_LEN * ROW_LEN;
    let x = i % ROW_LEN;
    let sect_height = COL_LEN_F.sqrt() as usize;
    let sect_y = i / (ROW_LEN * sect_height) * ROW_LEN * sect_height;
    let sect_x = i % ROW_LEN / sect_height * sect_height;
    let sect = sect_y + sect_x;

    [0, 1, 2, 9, 10, 11, 18, 19, 20]
        .iter()
        .map(move |&j| sect + j)
        .chain((0..9).map(move |j| y + j))
        .chain((0..9).map(move |j| x + ROW_LEN * j))
        .filter(move |&j| i != j)
        .collect()
}

fn state_reducer(neighbors: Vec<(usize, &SudokuCell)>, _: usize) -> CellState {
    let mut acc = CellState::ZERO;

    for (_, cell) in neighbors {
        acc |= cell.state();
    }

    acc
}

fn validate(state: &BoardState) {
    let mut errors = HashSet::new();
    for y in 0..COL_LEN {
        let mut seen = HashSet::with_capacity(9);

        for x in 0..ROW_LEN {
            if !seen.insert(state[ROW_LEN * y + x].value().unwrap()) {
                errors.insert(format!("Row {y}"));
            }
        }
    }

    for x in 0..ROW_LEN {
        let mut seen = HashSet::with_capacity(9);

        for y in 0..COL_LEN {
            if !seen.insert(state[ROW_LEN * y + x].value().unwrap()) {
                errors.insert(format!("Col {x}"));
            }
        }
    }

    for head in [0, 3, 6, 27, 30, 33, 54, 57, 60] {
        let mut seen = HashSet::with_capacity(9);

        for member in [0, 1, 2, 9, 10, 11, 18, 19, 20]
            .iter()
            .map(move |&j| head + j)
        {
            if !seen.insert(state[member].value().unwrap()) {
                errors.insert(format!("Sect {head}"));
            }
        }
    }

    if !errors.is_empty() {
        let mut errors = errors.iter().collect::<Vec<&String>>();
        errors.sort();
        panic!("Duplicate entries in the following: {:?}", errors);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut solver = SolverBuilder::new(neighbors, state_reducer)
        .state(parse(
            "6.....5.9.7..4..6.4........51.4...37....63.........9....29.8...........2.9.7.13..",
        )?)
        .build();

    let start_time = SystemTime::now();
    solver.solve();
    let elapsed = start_time.elapsed()?;

    print_board(&solver);
    println!("Took {:.4} ms", elapsed.as_secs_f64() * 1000.0);

    validate(solver.state());

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
            '1' => Ok(Cell::reduced(0)),
            '2' => Ok(Cell::reduced(1)),
            '3' => Ok(Cell::reduced(2)),
            '4' => Ok(Cell::reduced(3)),
            '5' => Ok(Cell::reduced(4)),
            '6' => Ok(Cell::reduced(5)),
            '7' => Ok(Cell::reduced(6)),
            '8' => Ok(Cell::reduced(7)),
            '9' => Ok(Cell::reduced(8)),
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
        match cell.value() {
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
