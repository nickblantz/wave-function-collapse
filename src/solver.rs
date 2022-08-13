use bitvec::{array::BitArray, order::Lsb0, view::BitViewSized};
use rand::{
    prelude::{thread_rng, SliceRandom, StdRng},
    SeedableRng,
};
use std::fmt::Debug;

use crate::cell::{Cell, Weights};

/// Represents the state of the solver at a given time
pub type SolverState<A, const N: usize, const S: usize> = [Cell<A, N>; S];

/// A function which returns cells adjacent to a given index
pub type Neighbors = fn(usize) -> Vec<usize>;

/// A function which returns a BitArray where each 1 represents a state
/// that the current tile cannot be in
pub type StateReducer<A, const N: usize> =
    fn(Vec<(usize, &Cell<A, N>)>, usize) -> BitArray<A, Lsb0>;

/// Solves a constraint problem using wave function collapse and backtracking
/// ```
/// use bitvec::{array::BitArray, order::Lsb0};
/// use wave_function_collapse::{cell::Cell, solver::SolverBuilder};
///
/// // The number of states your cell can collapse to
/// const STATES: usize = 8;
/// // The size of your solver
/// const BOARD_SIZE: usize = 16;
///
/// // The storage requirement of your state
/// type CellStorage = u16;
/// // The state wrapper of your cell
/// type CellState = BitArray<CellStorage, Lsb0>;
/// // The cell used in your solver
/// type MyCell = Cell<CellStorage, STATES>;
/// // The initial state of your solver
/// type SolverState = [MyCell; BOARD_SIZE];
///
/// // Returns a list of adjacent cells used to filter input to your state reducer
/// fn neighbors(i: usize) -> Vec<usize> {
///     todo!()
/// }
///
/// // Returns a cell state where each 1 represents a state that the current ith
/// // cannot be in
/// fn reducer(neighbors: Vec<(usize, &MyCell)>, i: usize) -> CellState {
///     todo!()
/// }
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let solver = SolverBuilder::new(neighbors, reducer)
///         .state([MyCell::default(); BOARD_SIZE])
///         .build();
///     Ok(())
/// }
/// ```
pub struct Solver<A: BitViewSized + Copy + Debug, const N: usize, const S: usize> {
    /// Current state of the board
    state: SolverState<A, N, S>,

    /// A stack of the historic board states
    history: Vec<SolverState<A, N, S>>,

    /// A function which returns a list of adjacent cells used to filter input
    /// to `reducer`
    neighbors: Neighbors,

    /// A function which returns a `BitArray` where each 1 represents a state that the current ith
    /// cannot be in
    reducer: StateReducer<A, N>,

    /// A function which returns the weight associated with a given state
    weights: Weights,

    /// Random noise for selecting and solving cells
    rng: StdRng,
}

impl<A: BitViewSized + Copy + Debug, const N: usize, const S: usize> Solver<A, N, S> {
    /// Returns the current state of the solver
    pub fn state(&self) -> &SolverState<A, N, S> {
        &self.state
    }

    /// Fills in every unsolved cell
    pub fn solve(&mut self) {
        let mut to_collapse = self.reduced();

        self.history.push(self.state.clone());
        self.propagate(to_collapse);

        while let Some(i) = self.lowest_entropy() {
            to_collapse = self.observe(i);
            self.propagate(to_collapse);
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

        self.history = vec![self.state.clone()];
    }

    /// Iterates over the board and propagate collapsed cells
    fn propagate(&mut self, to_collapse: Vec<usize>) {
        let mut to_collapse = to_collapse;
        let mut reduced = vec![];

        while !to_collapse.is_empty() {
            // println!("p: Collapsing: {:?}", to_collapse);
            for i in 0..S {
                if !self.state[i].is_unknown() {
                    continue;
                }

                let neighbors = (self.neighbors)(i)
                    .iter()
                    .filter(|&&j| !self.state[j].is_unknown())
                    .map(|&j| (j, &self.state[j]))
                    .collect::<Vec<(usize, &Cell<A, N>)>>();

                if neighbors.is_empty() {
                    continue;
                }

                let reductions = (self.reducer)(neighbors, i);

                if reductions.not_any() {
                    continue;
                }

                // print!("p: Reducing {i} to ");
                match self.state[i].reduce(reductions) {
                    Some(cell) => self.state[i] = cell,
                    None => {
                        // println!(" no possibilities");
                        let to_collapse = self.backtrack();
                        self.propagate(to_collapse);
                        return;
                    }
                }
                // println!("{:?}", self.state[i]);

                if self.state[i].is_reduced() {
                    reduced.push(i);
                }
            }

            for i in to_collapse {
                self.state[i] = self.state[i].collapse();
            }

            to_collapse = reduced;
            reduced = vec![];
        }
    }

    /// Randomly selects once cell with the lowest entropy
    fn lowest_entropy(&mut self) -> Option<usize> {
        let mut cells = self
            .state
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_unknown())
            // .inspect(|(_, c)| assert!(c.is_unknown()))
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
        // println!("o: Observing {i} {:?}", self.state[i]);
        match self.state[i].observe(self.weights, &mut self.rng) {
            Ok(cell) => {
                self.history.push({
                    let mut state = self.state.clone();
                    match state[i].reduce(cell.state()) {
                        Some(cell) => state[i] = cell,
                        _ => {}
                    }
                    // assert!(state[i].is_unknown());
                    state
                });
                self.state[i] = cell;
                vec![i]
            }
            Err(_) => self.backtrack(),
        }
    }

    fn backtrack(&mut self) -> Vec<usize> {
        // println!("backtracking!");
        // println!("{:?}", self.// print_board());
        match self.history.pop() {
            Some(state) => {
                self.state = state;
                // println!("{:?}", self.// print_board());
                // println!("{:?}", self.reduced());
                self.reduced()
            }
            None => {
                // println!("Input State:\n{:?}", self.print_board());
                // println!("retrying!");
                self.state = self.state;
                self.reduced()
            }
        }
    }

    fn reduced(&self) -> Vec<usize> {
        self.state
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_reduced())
            .map(|(i, _)| i)
            .collect::<Vec<usize>>()
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
}

/// The direction and distance to pan
pub enum Pan {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
}

pub struct SolverBuilder<A: BitViewSized + Copy + Debug, const N: usize, const S: usize> {
    seed: Option<u64>,
    state: Option<SolverState<A, N, S>>,
    neighbors: Neighbors,
    reducer: StateReducer<A, N>,
    weights: Option<Weights>,
}

fn uniform(_: &usize) -> usize {
    1
}

impl<A: BitViewSized + Copy + Debug, const N: usize, const S: usize> SolverBuilder<A, N, S> {
    pub fn new(neighbors: Neighbors, reducer: StateReducer<A, N>) -> Self {
        Self {
            seed: None,
            state: None,
            neighbors,
            reducer,
            weights: None,
        }
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn state(mut self, state: SolverState<A, N, S>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn weights(mut self, weights: Weights) -> Self {
        self.weights = Some(weights);
        self
    }

    pub fn build(self) -> Solver<A, N, S> {
        Solver {
            state: match self.state {
                Some(state) => state,
                None => [Cell::default(); S],
            },
            history: vec![],
            neighbors: self.neighbors,
            reducer: self.reducer,
            weights: match self.weights {
                Some(weights) => weights,
                None => uniform,
            },
            rng: match self.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_rng(thread_rng()).unwrap(),
            },
        }
    }
}
