# Wave Function Collapse
```
┌───────────────────────────────────────────────────────────────────┐
│ ┌─┬┐┌┬─┬┬─┐ ┌┬─┬┬┬┐  ┌┬┬┬───┐┌┬─┐┌┐ ┌┬┐ ┌─┬┬┬─┐  ┌───┬┐┌┬───┬─┐┌┐ │
│ ├┬┴┼┘├┬┤│┌┼─┘├┬┤└┼┤ ┌┤└┤│┌─┐│└┘┌┼┤└─┼┤├─┘ │└┼─┼─┐└┬┬┐├┴┘├┬──┤ ├┴┤ │
│ │└┬┤ │└┴┴┘│┌─┼┴┴─┴┘ └┴─┘└┴─┘└──┴┘└──┘└┴───┴─┘ └─┴─┴┘└┴┬─┤└─┬┼─┼┬┘ │
│ └─┤└┐└───┬┤│┌┘ ┌──┐  ┌──┐  ┌──┐ ┌────────┐ ┌────────┐ ├┐├──┤└─┼┴┐ │
│  ┌┤ └┐ ┌┬┤│├┘ ┌┤  │ ┌┤  │ ┌┤  │┌┤  ┌┬┬┬┬─┘┌┤  ┌┬┬┬┬┬┘ └┼┼─┐│┌─┼┬┘ │
│  └┴┬┬┘ └┼┤└┴┐ ││  │ ││  │ ││  │││  └┴┴┴┴┐ ││  ├┴┴┴┴┘ ┌┐└┴┐│├┘ ││  │
│   ┌┘├─┬┐├┤┌┐│ ││  │ ││  │ ││  │││  ┌┬┬┬┬┘ ││  │ ┌─┐┌─┤│┌┬┴┤└┬─┘├┐ │
│   │ ├┬┘││├┘└┘ ││  │ ││  │ ││  │││  │┴┴┴┘  ││  │ └─┴┘ └┴┼┴┐│┌┴┐┌┴┤ │
│ ┌┐│┌┼┤┌┼┤├─┐  ││  └─┴┘  └─┴┘  │││  │ ┌┐   ││  └─────┐ ┌┴─┘├┼┐└┴─┤ │
│ └┘└┘├┼┘│├┤┌┘  │├┬┬┬┬┬┬┬┬┬┬┬┬┬┬┘│├┬┬┘ └┴┬┐ │├┬┬┬┬┬┬┬┬┘ └┬┐┌┴┘└───┤ │
│ ┌┐  │├─┤│└┴┬┐ └┴┴┴┴┴┴┴┴┴┴┴┴┴┴┘ └┴┴┘  ┌─┼┤ └┴┴┴┴┴┴┴┴┘  ┌┘││ ┌┬─┬┬┘ │
│ ││┌┬┴┤┌┘│  │├─┐┌┐┌┬┐ ┌┐ ┌─┐┌┬┬─┐┌┐ ┌┬┘┌┤└┬┬┐┌┬┐┌─┐┌┐ ┌┼┬┤├┬┤└┬┴┴┐ │
│ │└┼┤ ││┌┘┌┐├┴┬┘└┼┤├┤┌┘│┌┴─┘├┤│┌┤└┼┐└┤┌┤│┌┼┘└┤│└┘ ├┘├┐└┘├┤│││ ├┐ │ │
│ └─┴┴─┘└┴─┴┘└─┘  └┴┴┴┘ └┘   └┘└┘└─┴┘ └┘└┘└┴──┴┘   └─┴┘  └┴┴┴┴─┴┴─┘ │
└───────────────────────────────────────────────────────────────────┘
```

This library implements a generalized wave function collapse algorithm with backtracking.

## Usage

```rust
use bitvec::{array::BitArray, order::Lsb0};
use wave_function_collapse::{cell::Cell, solver::SolverBuilder};

// The number of states your cell can collapse to
const STATES: usize = 8;
// The size of your solver
const BOARD_SIZE: usize = 16;

// The storage requirement of your state
type CellStorage = u16;
// The state wrapper of your cell
type CellState = BitArray<CellStorage, Lsb0>;
// The cell used in your solver
type MyCell = Cell<CellStorage, STATES>;
// The initial state of your solver
type SolverState = [MyCell; BOARD_SIZE];

// Returns a list of adjacent cells used to filter input to your state reducer
fn neighbors(i: usize) -> Vec<usize> {
    todo!()
}

// Returns a cell state where each 1 represents a state that the current ith
// cannot be in
fn reducer(neighbors: Vec<(usize, &MyCell)>, i: usize) -> CellState {
    todo!()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let solver = SolverBuilder::new(neighbors, reducer)
        .state([MyCell::default(); BOARD_SIZE])
        .build();
    Ok(())
}
```
