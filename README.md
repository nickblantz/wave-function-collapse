# Wave Function Collapse

This library implements a generalized wave function collapse algorithm.

## Usage

```rust
use bitvec::{array::BitArray, order::Lsb0};
use wave_function_collapse::{cell::Cell, solver::Solver};

// The number of states your cell can collapse to
const STATES: usize = 8;

// The size of your solver
const BOARD_SIZE: usize = 16;

// The storage requirement of your state
type CellStorage = [u16; 1];

// The state wrapper of your cell
type CellState = BitArray<CellStorage, Lsb0>;

// The cell used in your solver
type MyCell = Cell<CellStorage, STATES>;

// The initial state of your solver
type SolverState = [MyCell; BOARD_SIZE];

// Returns a list of adjacent cells used to filter input to your state reducer 
fn adjacencies(i: usize) -> Vec<usize> {
    todo!()
}

// Returns a cell state where each 1 represents a state that the current ith
// cannot be in
fn state_reducer(cell: (usize, &MyCell), i: usize) -> CellState {
    todo!()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = [MyCell::default(); BOARD_SIZE];
    let mut solver = Solver::new(state, adjacencies, state_reducer);
    Ok(())
}
```

# Examples

```
cargo run --release --example path

┌┼┘├┘┌┬┐
├┴─┘ ├┘└
└┬┬┐ └─┐
┬┘│├┐  │
└┬┴┘└──┼
 ├──┐ ┌┘
┬┤ ┌┼┬┼┬
└┴┐│├┤└┘

Took 0.5479 ms
```

```
cargo run --release --example sudoku

6 8 3   2 1 7   5 4 9
1 7 5   3 4 9   2 6 8
4 2 9   5 8 6   7 1 3

5 1 8   4 9 2   6 3 7
9 4 7   8 6 3   1 2 5
2 3 6   1 7 5   9 8 4

3 6 2   9 5 8   4 7 1
7 5 1   6 3 4   8 9 2
8 9 4   7 2 1   3 5 6

Took 1.7011 ms
```
