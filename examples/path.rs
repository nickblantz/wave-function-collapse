use bitvec::{array::BitArray, order::Lsb0};
use std::{
    cmp::Ordering,
    fmt, thread,
    time::{Duration, SystemTime},
};

use wave_function_collapse::{
    cell::Cell,
    solver::{Pan, Solver, SolverBuilder},
};

const STATES: usize = 12;
const ROW_LEN: usize = 69;
const COL_LEN: usize = 16;
const BOARD_SIZE: usize = ROW_LEN * COL_LEN;

type CellStorage = u16;
type CellState = BitArray<CellStorage, Lsb0>;
type PathCell = Cell<CellStorage, STATES>;
type BoardState = [PathCell; BOARD_SIZE];

fn neighbors(i: usize) -> Vec<usize> {
    let mut neighbors = vec![];
    let x = i % ROW_LEN;
    let y = i / ROW_LEN;
    let left = x > 0;
    let right = x < ROW_LEN - 1;
    let top = y > 0;
    let bottom = y < COL_LEN - 1;

    if left {
        neighbors.push(y * ROW_LEN + x - 1);
    }

    if right {
        neighbors.push(y * ROW_LEN + x + 1);
    }

    if top {
        neighbors.push((y - 1) * ROW_LEN + x);
    }

    if bottom {
        neighbors.push((y + 1) * ROW_LEN + x);
    }

    neighbors
}

fn state_reducer(neighbors: Vec<(usize, &PathCell)>, i: usize) -> CellState {
    const LEFT_CONNECTED: u16 = 0b0011_0110_1101;
    const LEFT_DISCONNECTED: u16 = 0b1100_1001_0010;
    const LEFT_REDUCTIONS: [CellStorage; 12] = [
        LEFT_CONNECTED,    // ┐
        LEFT_DISCONNECTED, // └
        LEFT_DISCONNECTED, // ┴
        LEFT_DISCONNECTED, // ┬
        LEFT_DISCONNECTED, // ├
        LEFT_DISCONNECTED, // ─
        LEFT_DISCONNECTED, // ┼
        LEFT_CONNECTED,    // │
        LEFT_CONNECTED,    // ┤
        LEFT_CONNECTED,    // ┘
        LEFT_DISCONNECTED, // ┌
        LEFT_CONNECTED,    //
    ];
    const RIGHT_CONNECTED: u16 = 0b0100_0111_1110;
    const RIGHT_DISCONNECTED: u16 = 0b1011_1000_0001;
    const RIGHT_REDUCTIONS: [CellStorage; 12] = [
        RIGHT_DISCONNECTED, // ┐
        RIGHT_CONNECTED,    // └
        RIGHT_DISCONNECTED, // ┴
        RIGHT_DISCONNECTED, // ┬
        RIGHT_CONNECTED,    // ├
        RIGHT_DISCONNECTED, // ─
        RIGHT_DISCONNECTED, // ┼
        RIGHT_CONNECTED,    // │
        RIGHT_DISCONNECTED, // ┤
        RIGHT_DISCONNECTED, // ┘
        RIGHT_CONNECTED,    // ┌
        RIGHT_CONNECTED,    //
    ];
    const TOP_CONNECTED: u16 = 0b0011_1101_0110;
    const TOP_DISCONNECTED: u16 = 0b1100_0010_1001;
    const TOP_REDUCTIONS: [CellStorage; 12] = [
        TOP_DISCONNECTED, // ┐
        TOP_CONNECTED,    // └
        TOP_CONNECTED,    // ┴
        TOP_DISCONNECTED, // ┬
        TOP_DISCONNECTED, // ├
        TOP_CONNECTED,    // ─
        TOP_DISCONNECTED, // ┼
        TOP_DISCONNECTED, // │
        TOP_DISCONNECTED, // ┤
        TOP_CONNECTED,    // ┘
        TOP_DISCONNECTED, // ┌
        TOP_CONNECTED,    //
    ];
    const BOTTOM_CONNECTED: u16 = 0b0101_1101_1001;
    const BOTTOM_DISCONNECTED: u16 = 0b1010_0010_0110;
    const BOTTOM_REDUCTIONS: [CellStorage; 12] = [
        BOTTOM_CONNECTED,    // ┐
        BOTTOM_DISCONNECTED, // └
        BOTTOM_DISCONNECTED, // ┴
        BOTTOM_CONNECTED,    // ┬
        BOTTOM_DISCONNECTED, // ├
        BOTTOM_CONNECTED,    // ─
        BOTTOM_DISCONNECTED, // ┼
        BOTTOM_DISCONNECTED, // │
        BOTTOM_DISCONNECTED, // ┤
        BOTTOM_DISCONNECTED, // ┘
        BOTTOM_CONNECTED,    // ┌
        BOTTOM_CONNECTED,    //
    ];

    let mut acc = CellState::ZERO;

    for cell in neighbors {
        let (j, cell) = cell;
        let ix = i % ROW_LEN;
        let iy = i / ROW_LEN;
        let jx = j % ROW_LEN;
        let jy = j / ROW_LEN;
        let result = cell
            .value()
            .expect(&format!("Cell {} was uncollapsed: {}", j, cell.state()));

        acc |= match (ix.cmp(&jx), iy.cmp(&jy)) {
            (Ordering::Greater, Ordering::Equal) => CellState::new(LEFT_REDUCTIONS[result]),
            (Ordering::Less, Ordering::Equal) => CellState::new(RIGHT_REDUCTIONS[result]),
            (Ordering::Equal, Ordering::Greater) => CellState::new(TOP_REDUCTIONS[result]),
            (Ordering::Equal, Ordering::Less) => CellState::new(BOTTOM_REDUCTIONS[result]),
            (_, _) => unreachable!(),
        };
    }

    acc
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("examples/wfc.txt")?;
    let mut solver = SolverBuilder::new(neighbors, state_reducer)
        .state(parse(&contents)?)
        .seed(5)
        .build();

    solver.solve();
    print_board(&solver);

    for _ in 0..8 {
        let start_time = SystemTime::now();
        solver.pan(Pan::Down(8), ROW_LEN);
        solver.solve();
        bottom_rows(8, &solver);
        let elapsed = start_time.elapsed()?;
        // println!("took {} ms", elapsed.as_millis());
        thread::sleep(
            Duration::from_millis(100)
                .checked_sub(elapsed)
                .unwrap_or(Duration::from_millis(0)),
        );
    }

    Ok(())
}

fn format_cell(cell: &PathCell) -> String {
    match cell.value() {
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

fn bottom_rows(rows: usize, solver: &Solver<CellStorage, STATES, BOARD_SIZE>) {
    let state = solver.state();
    for i in (ROW_LEN * (COL_LEN - rows))..state.len() {
        let cell = &state[i];
        print!("{}", format_cell(&cell));

        if (i + 1) % ROW_LEN == 0 {
            println!();
        }
    }
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
    let raw = raw
        .chars()
        .filter(|&c| c != '\r' && c != '\n')
        .collect::<Vec<char>>();
    if raw.len() != BOARD_SIZE {
        return Err(ParseError::InvalidSize(raw.len()));
    }

    let char_map = |c: (usize, &char)| -> Result<PathCell, ParseError> {
        match c.1 {
            '┐' => Ok(Cell::reduced(0)),
            '└' => Ok(Cell::reduced(1)),
            '┴' => Ok(Cell::reduced(2)),
            '┬' => Ok(Cell::reduced(3)),
            '├' => Ok(Cell::reduced(4)),
            '─' => Ok(Cell::reduced(5)),
            '┼' => Ok(Cell::reduced(6)),
            '│' => Ok(Cell::reduced(7)),
            '┤' => Ok(Cell::reduced(8)),
            '┘' => Ok(Cell::reduced(9)),
            '┌' => Ok(Cell::reduced(10)),
            ' ' => Ok(Cell::reduced(11)),
            '.' => Ok(Cell::default()),
            _ => Err(ParseError::InvalidInput(c.0, *c.1)),
        }
    };

    match TryInto::<BoardState>::try_into(
        raw.iter()
            .enumerate()
            .map(char_map)
            .collect::<Result<Vec<PathCell>, ParseError>>()?,
    ) {
        Ok(state) => Ok(state),
        Err(_) => Err(ParseError::InternalError),
    }
}
