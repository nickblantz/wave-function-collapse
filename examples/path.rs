use bitvec::{array::BitArray, order::Lsb0};
use std::{fmt, time::SystemTime};

use wave_function_collapse::{cell::Cell, solver::Solver};

const STATES: usize = 12;
const ROW_LEN: usize = 8;
const COL_LEN: usize = 8;
const BOARD_SIZE: usize = ROW_LEN * COL_LEN;
type CellStorage = [u16; 1];
type CellArray = BitArray<CellStorage, Lsb0>;
type PathCell = Cell<CellStorage, STATES>;
type BoardState = [PathCell; BOARD_SIZE];

const LEFT_CONNECTED: [usize; 7] = [0, 2, 3, 5, 6, 8, 9];
const LEFT_DISCONNECTED: [usize; 5] = [1, 4, 7, 10, 11];
const RIGHT_CONNECTED: [usize; 7] = [1, 2, 3, 4, 5, 6, 10];
const RIGHT_DISCONNECTED: [usize; 5] = [0, 7, 8, 9, 11];
const TOP_CONNECTED: [usize; 7] = [1, 2, 4, 6, 7, 8, 9];
const TOP_DISCONNECTED: [usize; 5] = [0, 3, 5, 10, 11];
const BOTTOM_CONNECTED: [usize; 7] = [0, 3, 4, 6, 7, 8, 10];
const BOTTOM_DISCONNECTED: [usize; 5] = [1, 2, 5, 9, 11];

fn adjacencies(i: usize) -> Vec<usize> {
    let mut adjacencies = vec![];
    let x = i % ROW_LEN;
    let y = i / COL_LEN;

    // left
    if x > 0 {
        adjacencies.push(y * COL_LEN + x - 1);
    }

    // right
    if x < ROW_LEN - 1 {
        adjacencies.push(y * COL_LEN + x + 1);
    }

    // top
    if y > 0 {
        adjacencies.push((y - 1) * COL_LEN + x);
    }

    // bottom
    if y < COL_LEN - 1 {
        adjacencies.push((y + 1) * COL_LEN + x);
    }

    adjacencies
}

fn state_reducer(acc: CellArray, cell: &PathCell, i: usize, j: usize) -> CellArray {
    let result = cell.result().unwrap();
    let mut possibilities = CellArray::ZERO;
    let ix = i % ROW_LEN;
    let iy = i / ROW_LEN;
    let jx = j % COL_LEN;
    let jy = j / COL_LEN;

    // left
    if jx + 1 == ix && jy == iy {
        if RIGHT_CONNECTED.contains(&result) {
            for state in LEFT_DISCONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        } else {
            for state in LEFT_CONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        }
    }

    // right
    if jx == ix + 1 && jy == iy {
        if LEFT_CONNECTED.contains(&result) {
            for state in RIGHT_DISCONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        } else {
            for state in RIGHT_CONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        }
    }

    // top
    if jx == ix && jy + 1 == iy {
        if BOTTOM_CONNECTED.contains(&result) {
            for state in TOP_DISCONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        } else {
            for state in TOP_CONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        }
    }

    // bottom
    if jx == ix && jy == iy + 1 {
        if TOP_CONNECTED.contains(&result) {
            for state in BOTTOM_DISCONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        } else {
            for state in BOTTOM_CONNECTED {
                possibilities |= PathCell::from(state).state();
            }
        }
    }

    acc | possibilities
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = parse("┌......┐................................................└......┘")?;
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
    let raw = raw.chars().collect::<Vec<char>>();
    if raw.len() != BOARD_SIZE {
        return Err(ParseError::InvalidSize(raw.len()));
    }

    let char_map = |c: (usize, &char)| -> Result<PathCell, ParseError> {
        match c.1 {
            '┐' => Ok(Cell::from(0)),
            '└' => Ok(Cell::from(1)),
            '┴' => Ok(Cell::from(2)),
            '┬' => Ok(Cell::from(3)),
            '├' => Ok(Cell::from(4)),
            '─' => Ok(Cell::from(5)),
            '┼' => Ok(Cell::from(6)),
            '│' => Ok(Cell::from(7)),
            '┤' => Ok(Cell::from(8)),
            '┘' => Ok(Cell::from(9)),
            '┌' => Ok(Cell::from(10)),
            ' ' => Ok(Cell::from(11)),
            '.' => Ok(Cell::default()),
            _ => Err(ParseError::InvalidInput(c.0, *c.1)),
        }
    };

    match TryInto::<[PathCell; BOARD_SIZE]>::try_into(
        raw.iter()
            .enumerate()
            .map(char_map)
            .collect::<Result<Vec<PathCell>, ParseError>>()?,
    ) {
        Ok(state) => Ok(state),
        Err(_) => Err(ParseError::InternalError),
    }
}

fn format_cell(cell: &PathCell) -> String {
    match cell.result() {
        Some(0) => format!("{}", '┐'),
        Some(1) => format!("{}", '└'),
        Some(2) => format!("{}", '┴'),
        Some(3) => format!("{}", '┬'),
        Some(4) => format!("{}", '├'),
        Some(5) => format!("{}", '─'),
        Some(6) => format!("{}", '┼'),
        Some(7) => format!("{}", '│'),
        Some(8) => format!("{}", '┤'),
        Some(9) => format!("{}", '┘'),
        Some(10) => format!("{}", '┌'),
        Some(11) => format!("{}", ' '),
        Some(_) | None => format!("({}) ", cell.entropy()),
    }
}

fn print_board(solver: &Solver<CellStorage, STATES, BOARD_SIZE>) {
    let state = solver.state();
    for i in 0..state.len() {
        let cell = &state[i];
        print!("{}", format_cell(&cell));

        if (i + 1) % ROW_LEN == 0 {
            println!();
        }
    }
}
