use bitvec::{array::BitArray, order::Lsb0, view::BitViewSized};
use rand::prelude::{SliceRandom, ThreadRng};

/// A BitArray where each 1 represnts a state that the cell could be in
pub type CellState<A> = BitArray<A, Lsb0>;

#[derive(Clone, Copy)]
pub struct Cell<A: BitViewSized + Copy, const N: usize> {
    /// A BitArray where each 1 represnts a state that the cell could be in
    state: CellState<A>,
    
    /// Is Some when the cell has been fully collapsed
    result: Option<usize>,
}

impl<A: BitViewSized + Copy, const N: usize> Default for Cell<A, N> {
    fn default() -> Self {
        Self {
            state: {
                let mut bits = BitArray::ZERO;
                for i in 0..N {
                    bits.set(i, true);
                }
                bits
            },
            result: None,
        }
    }
}

impl<A: BitViewSized + Copy, const N: usize> From<usize> for Cell<A, N> {
    fn from(n: usize) -> Self {
        Self {
            state: {
                let mut bits = BitArray::ZERO;
                bits.set(n, true);
                bits
            },
            result: Some(n),
        }
    }
}

impl<A: BitViewSized + Copy, const N: usize> Cell<A, N> {
    /// Takes a BitArray where each 1 represents a state the cell cannot be in
    /// and removes those states from the current cell
    pub fn collapse(&mut self, state: CellState<A>) {
        self.state = self.state & !state;
        self.result = match state.count_zeros() {
            1 => Some(self.state.first_one().unwrap()),
            _ => None,
        };
    }

    /// Randomly selects a possible state
    pub fn solve_rng(&self, rng: &mut ThreadRng) -> Option<Self> {
        self.state
            .iter_ones()
            .collect::<Vec<usize>>()
            .choose(rng)
            .map(ToOwned::to_owned)
            .map(Self::from)
    }

    /// Updates the result for an fully collapsed cell
    pub fn solve(&self) -> Self {
        assert!(self.entropy() == 1);

        Self::from(self.state.first_one().unwrap())
    }

    /// The number of possible states in the cell's superposition
    pub fn entropy(&self) -> usize {
        self.state.count_ones()
    }

    /// The result of the cell
    pub fn result(&self) -> Option<usize> {
        self.result
    }

    /// The state of the cell
    pub fn state(&self) -> CellState<A> {
        self.state
    }
}
