use bitvec::{array::BitArray, order::Lsb0};
use std::{
    cmp::Ordering,
    thread,
    time::{Duration, SystemTime},
};

use wave_function_collapse::{
    cell::Cell,
    solver::{Pan, Solver},
};

const STATES: usize = 12;
const ROW_LEN: usize = 64;
const COL_LEN: usize = 16;
const BOARD_SIZE: usize = ROW_LEN * COL_LEN;

type CellStorage = [u16; 1];
type CellState = BitArray<CellStorage, Lsb0>;
type PathCell = Cell<CellStorage, STATES>;

fn adjacencies(i: usize) -> Vec<usize> {
    let mut adjacencies = vec![];
    let x = i % ROW_LEN;
    let y = i / ROW_LEN;

    // left
    if x > 0 {
        adjacencies.push(y * ROW_LEN + x - 1);
    }

    // right
    if x < ROW_LEN - 1 {
        adjacencies.push(y * ROW_LEN + x + 1);
    }

    // top
    if y > 0 {
        adjacencies.push((y - 1) * ROW_LEN + x);
    }

    // bottom
    if y < COL_LEN - 1 {
        adjacencies.push((y + 1) * ROW_LEN + x);
    }

    adjacencies
}

fn state_reducer(cell: (usize, &PathCell), i: usize) -> CellState {
    const LEFT_CONNECTED: u16 = 0b0011_0110_1101;
    const LEFT_DISCONNECTED: u16 = 0b1100_1001_0010;
    const LEFT_REDUCTIONS: [CellStorage; 12] = [
        [LEFT_CONNECTED],    // ┐
        [LEFT_DISCONNECTED], // └
        [LEFT_DISCONNECTED], // ┴
        [LEFT_DISCONNECTED], // ┬
        [LEFT_DISCONNECTED], // ├
        [LEFT_DISCONNECTED], // ─
        [LEFT_DISCONNECTED], // ┼
        [LEFT_CONNECTED],    // │
        [LEFT_CONNECTED],    // ┤
        [LEFT_CONNECTED],    // ┘
        [LEFT_DISCONNECTED], // ┌
        [LEFT_CONNECTED],    //
    ];

    const RIGHT_CONNECTED: u16 = 0b0100_0111_1110;
    const RIGHT_DISCONNECTED: u16 = 0b1011_1000_0001;
    const RIGHT_REDUCTIONS: [CellStorage; 12] = [
        [RIGHT_DISCONNECTED], // ┐
        [RIGHT_CONNECTED],    // └
        [RIGHT_DISCONNECTED], // ┴
        [RIGHT_DISCONNECTED], // ┬
        [RIGHT_CONNECTED],    // ├
        [RIGHT_DISCONNECTED], // ─
        [RIGHT_DISCONNECTED], // ┼
        [RIGHT_CONNECTED],    // │
        [RIGHT_DISCONNECTED], // ┤
        [RIGHT_DISCONNECTED], // ┘
        [RIGHT_CONNECTED],    // ┌
        [RIGHT_CONNECTED],    //
    ];

    const TOP_CONNECTED: u16 = 0b0011_1101_0110;
    const TOP_DISCONNECTED: u16 = 0b1100_0010_1001;
    const TOP_REDUCTIONS: [CellStorage; 12] = [
        [TOP_DISCONNECTED], // ┐
        [TOP_CONNECTED],    // └
        [TOP_CONNECTED],    // ┴
        [TOP_DISCONNECTED], // ┬
        [TOP_DISCONNECTED], // ├
        [TOP_CONNECTED],    // ─
        [TOP_DISCONNECTED], // ┼
        [TOP_DISCONNECTED], // │
        [TOP_DISCONNECTED], // ┤
        [TOP_CONNECTED],    // ┘
        [TOP_DISCONNECTED], // ┌
        [TOP_CONNECTED],    //
    ];

    const BOTTOM_CONNECTED: u16 = 0b0101_1101_1001;
    const BOTTOM_DISCONNECTED: u16 = 0b1010_0010_0110;
    const BOTTOM_REDUCTIONS: [CellStorage; 12] = [
        [BOTTOM_CONNECTED],    // ┐
        [BOTTOM_DISCONNECTED], // └
        [BOTTOM_DISCONNECTED], // ┴
        [BOTTOM_CONNECTED],    // ┬
        [BOTTOM_DISCONNECTED], // ├
        [BOTTOM_CONNECTED],    // ─
        [BOTTOM_DISCONNECTED], // ┼
        [BOTTOM_DISCONNECTED], // │
        [BOTTOM_DISCONNECTED], // ┤
        [BOTTOM_DISCONNECTED], // ┘
        [BOTTOM_CONNECTED],    // ┌
        [BOTTOM_CONNECTED],    //
    ];
    let (j, cell) = cell;
    let ix = i % ROW_LEN;
    let iy = i / ROW_LEN;
    let jx = j % ROW_LEN;
    let jy = j / ROW_LEN;
    let result = cell
        .result()
        .expect(&format!("Cell {} was uncollapsed: {}", j, cell.state()));

    let reductions = match (ix.cmp(&jx), iy.cmp(&jy)) {
        (Ordering::Greater, Ordering::Equal) => CellState::new(LEFT_REDUCTIONS[result]),
        (Ordering::Less, Ordering::Equal) => CellState::new(RIGHT_REDUCTIONS[result]),
        (Ordering::Equal, Ordering::Greater) => CellState::new(TOP_REDUCTIONS[result]),
        (Ordering::Equal, Ordering::Less) => CellState::new(BOTTOM_REDUCTIONS[result]),
        (_, _) => unreachable!(),
    };

    reductions
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = [PathCell::default(); BOARD_SIZE];
    state[0].collapse(!PathCell::from(10).state());
    state[ROW_LEN - 1].collapse(!PathCell::from(0).state());
    let mut solver = Solver::new(state, adjacencies, state_reducer);

    solver.solve();
    print_board(&solver);

    for _ in 0..32 {
        let start_time = SystemTime::now();
        solver.pan(Pan::Down(8), ROW_LEN);
        solver.solve();
        let elapsed = start_time.elapsed()?;
        thread::sleep(
            Duration::from_millis(250)
                .checked_sub(elapsed)
                .unwrap_or(Duration::from_millis(0)),
        );
        bottom_rows(8, &solver);
    }

    Ok(())
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

fn bottom_rows(rows: usize, solver: &Solver<CellStorage, STATES, BOARD_SIZE>) {
    let state = solver.state();
    for i in (ROW_LEN * rows)..state.len() {
        let cell = &state[i];
        print!("{}", format_cell(&cell));

        if (i + 1) % ROW_LEN == 0 {
            println!();
        }
    }
}
