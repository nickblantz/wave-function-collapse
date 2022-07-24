use bitvec::{array::BitArray, order::Lsb0, view::BitViewSized};
use rand::prelude::{thread_rng, SliceRandom, ThreadRng};

use crate::cell::Cell;

/// Represents the state of the solver at a given time
pub type SolverState<A, const N: usize, const S: usize> = [Cell<A, N>; S];

/// A function which returns cells adjacent to a given index
pub type Adjacencies = fn(usize) -> Vec<usize>;

/// A function which returns a BitArray where each 1 represents a state
/// that the current tile cannot be in
pub type StateReducer<A, const N: usize> = fn((usize, &Cell<A, N>), usize) -> BitArray<A, Lsb0>;

/// Solves a constraint problem using wave function collapse and backtracking
/// ```
/// use bitvec::{array::BitArray, order::Lsb0};
/// use wave_function_collapse::{cell::Cell, solver::Solver};
/// 
/// // The number of states your cell can collapse to
/// const STATES: usize = 8;
/// 
/// // The size of your solver
/// const BOARD_SIZE: usize = 16;
/// 
/// // The storage requirement of your state
/// type CellStorage = [u16; 1];
/// 
/// // The state wrapper of your cell
/// type CellState = BitArray<CellStorage, Lsb0>;
/// 
/// // The cell used in your solver
/// type MyCell = Cell<CellStorage, STATES>;
/// 
/// // The initial state of your solver
/// type SolverState = [MyCell; BOARD_SIZE];
/// 
/// // Returns a list of adjacent cells used to filter input to your state reducer 
/// fn adjacencies(i: usize) -> Vec<usize> {
///     todo!()
/// }
/// 
/// // Returns a cell state where each 1 represents a state that the current ith
/// // cannot be in
/// fn state_reducer(cell: (usize, &MyCell), i: usize) -> CellState {
///     todo!()
/// }
/// 
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let state = [MyCell::default(); BOARD_SIZE];
///     let mut solver = Solver::new(state, adjacencies, state_reducer);
///     Ok(())
/// }
/// ```
pub struct Solver<A: BitViewSized + Copy, const N: usize, const S: usize> {
    /// Current state of the board
    state: SolverState<A, N, S>,

    /// A stack of the historic board states
    history: Vec<SolverState<A, N, S>>,

    /// A function which returns a list of adjacent cells used to filter input
    /// to `state_reducer`
    adjacencies: Adjacencies,

    /// A function which returns a `BitArray` where each 1 represents a state that the current ith
    /// cannot be in
    state_reducer: StateReducer<A, N>,

    /// Random noise for selecting and solving cells
    rng: ThreadRng,
}

impl<A: BitViewSized + Copy, const N: usize, const S: usize> Solver<A, N, S> {
    /// Creates a new solver
    pub fn new(
        state: SolverState<A, N, S>,
        adjacencies: Adjacencies,
        state_reducer: StateReducer<A, N>,
    ) -> Self {
        Self {
            state,
            history: vec![],
            adjacencies,
            state_reducer,
            rng: thread_rng(),
        }
    }

    /// Returns the current state of the solver
    pub fn state(&self) -> &SolverState<A, N, S> {
        &self.state
    }

    /// Iterates over the board and collapse cells
    fn collapse(&mut self, updates: Vec<usize>) {
        let mut updates = updates;

        while !updates.is_empty() {
            let mut new_updates = vec![];

            for i in 0..S {
                if self.state[i].result().is_some() {
                    continue;
                }

                let reduction = (self.adjacencies)(i)
                    .iter()
                    .filter(|&j| updates.iter().find(|&k| j == k).is_some())
                    .map(|&j| (j, &self.state[j]))
                    .fold(BitArray::ZERO, |acc, cell| {
                        acc | (self.state_reducer)(cell, i)
                    });

                if reduction.count_ones() == 0 {
                    continue;
                }

                self.state[i].collapse(reduction);

                if self.state[i].result().is_some() {
                    new_updates.push(i);
                }
            }

            for &i in &updates {
                self.state[i] = self.state[i].solve();
            }
            updates = new_updates;
        }
    }

    /// Randomly selects once cell with the lowest entropy
    fn lowest_entropy(&mut self) -> Option<usize> {
        let mut cells = self
            .state
            .iter()
            .enumerate()
            .filter(|(_, c)| c.result().is_none())
            .collect::<Vec<(usize, &Cell<A, N>)>>();

        if cells.is_empty() {
            return None;
        }

        cells.sort_by(|&(_, c1), &(_, c2)| c1.entropy().cmp(&c2.entropy()));

        let least_entropy = cells[0].1.entropy();

        cells
            .iter()
            .take_while(|(_, c)| c.entropy() == least_entropy)
            .map(|&(i, _)| i)
            .collect::<Vec<usize>>()
            .choose(&mut self.rng)
            .map(ToOwned::to_owned)
    }

    /// Tries to solve a cell, if there is no solution it resets the board
    fn observe(&mut self, i: usize) -> Vec<usize> {
        match self.state[i].solve_rng(&mut self.rng) {
            Some(cell) => {
                self.history.push({
                    let mut state = self.state.clone();
                    state[i].collapse(cell.state());
                    state
                });
                self.state[i] = cell;
                vec![i]
            }
            None => {
                self.state = self.history.pop().unwrap();
                vec![]
            }
        }
    }

    /// Fills in every unsolved cell
    pub fn solve(&mut self) {
        let mut updates = self
            .state
            .iter()
            .enumerate()
            .filter(|(_, c)| c.result().is_some())
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        self.collapse(updates);

        while let Some(i) = self.lowest_entropy() {
            updates = self.observe(i);
            self.collapse(updates);
        }
    }

    fn pan_direction<'a, I: Iterator<Item = usize>>(
        &mut self,
        iter: I,
        predicate: &'a dyn Fn(usize) -> bool,
        accessor: &'a dyn Fn(usize) -> usize,
    ) {
        for i in iter {
            self.state[i] = if predicate(i) {
                self.state[accessor(i)]
            } else {
                Cell::<A, N>::default()
            };
        }
    }

    /// Pans the solver, shifting the entire state by the distance in `Pan`
    pub fn pan(&mut self, pan: Pan, row_len: usize) {
        match pan {
            Pan::Left(distance) => {
                self.pan_direction((0..S).rev(), &|i| i % row_len > distance, &|i| i - distance)
            }
            Pan::Right(distance) => {
                self.pan_direction(0..S, &|i| row_len - i % row_len > distance, &|i| {
                    i + distance
                })
            }
            Pan::Up(distance) => {
                self.pan_direction((0..S).rev(), &|i| i / row_len > distance, &|i| {
                    (i / row_len - distance) * row_len + i % row_len
                })
            }
            Pan::Down(distance) => {
                self.pan_direction(0..S, &|i| S / row_len - i / row_len > distance, &|i| {
                    (i / row_len + distance) * row_len + i % row_len
                })
            }
        }
    }
}

/// The direction and distance to pan
pub enum Pan {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
}
