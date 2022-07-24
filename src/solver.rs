use bitvec::{array::BitArray, order::Lsb0, view::BitViewSized};
use rand::prelude::{thread_rng, SliceRandom, ThreadRng};

use crate::cell::Cell;

/// Represents the state of the solver at a given time
pub type SolverState<A, const N: usize, const S: usize> = [Cell<A, N>; S];

/// A function which returns cells adjacent to a given index
pub type Adjacencies = fn(usize) -> Vec<usize>;

/// A function which returns a BitArray where each 1 represents a state
/// that the current tile cannot be in
pub type StateReducer<A, const N: usize> = fn(BitArray<A, Lsb0>, &Cell<A, N>, usize, usize) -> BitArray<A, Lsb0>;

/// Solves a constraint problem using wave function collapse and backtracking
/// ```
/// use bitvec::{array::BitArray, order::Lsb0};
/// use wave_function_collapse::{cell::Cell, solver::Solver};
/// 
/// const STATES: usize = 8;
/// const SIDE_LEN: usize = 3;
/// const BOARD_SIZE: usize = SIDE_LEN * SIDE_LEN;
/// type CellStorage = [u16; 1];
/// type CellArray = BitArray<CellStorage, Lsb0>;
/// type MyCell = Cell<CellStorage, STATES>;
/// type BoardState = [MyCell; BOARD_SIZE];
/// 
/// fn adjacencies(i: usize) -> Vec<usize> {
///     todo!()
/// }
/// 
/// fn state_reducer(acc: CellArray, cell: &MyCell, _: usize, _: usize) -> CellArray {
///     todo!()
/// }
/// 
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let state: BoardState = [MyCell::default(); BOARD_SIZE];
///     let mut solver = Solver::new(state, adjacencies, state_reducer);
///     Ok(())
/// }
/// ```
pub struct Solver<A: BitViewSized + Copy, const N: usize, const S: usize> {
    /// Current state of the board
    state: SolverState<A, N, S>,

    /// A stack of the historic board states
    history: Vec<SolverState<A, N, S>>,
    
    /// A function which returns cells adjacent to a given index
    adjacencies: Adjacencies,
    
    /// A function which returns a BitArray where each 1 represents a state
    /// that the current tile cannot be in
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

            for i in 0..self.state.len() {
                if self.state[i].result().is_some() {
                    continue;
                }

                let state = (self.adjacencies)(i)
                    .iter()
                    .filter(|&j| updates.iter().find(|&k| j == k).is_some())
                    .map(|&j| (j, &self.state[j]))
                    .fold(BitArray::<A, Lsb0>::ZERO, |acc, (j, cell)| {
                        (self.state_reducer)(acc, cell, i, j)
                    });

                if state.count_ones() == 0 {
                    continue;
                }

                self.state[i].collapse(state);

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
        // panic!("stop");

        while let Some(i) = self.lowest_entropy() {
            updates = self.observe(i);
            self.collapse(updates);
        }
    }
}
