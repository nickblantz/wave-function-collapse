use bitvec::{array::BitArray, order::Lsb0, view::BitViewSized};
use rand::{
    distributions::WeightedError,
    prelude::{SliceRandom, StdRng},
};
use std::fmt::Debug;

/// A BitArray where each 1 represnts a state that the cell could be in
pub type CellState<A> = BitArray<A, Lsb0>;

/// A function which returns the weight associated with a given state
pub type Weights = fn(&usize) -> usize;

#[derive(Clone, Copy)]
pub enum Cell<A: BitViewSized + Clone + Debug, const N: usize> {
    Unknown(CellState<A>),
    Reduced(CellState<A>, usize),
    Collapsed(CellState<A>, usize),
}

impl<A: BitViewSized + Clone + Debug, const N: usize> Debug for Cell<A, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Unknown(state) => writeln!(f, "{}", state)?,
            Self::Reduced(_, n) => writeln!(f, "({})", n)?,
            Self::Collapsed(_, n) => writeln!(f, "[{}]", n)?,
        }
        Ok(())
    }
}

impl<A: BitViewSized + Clone + Debug, const N: usize> Default for Cell<A, N> {
    fn default() -> Self {
        Self::Unknown({
            let mut bits = BitArray::ZERO;
            for i in 0..N {
                bits.set(i, true);
            }
            bits
        })
    }
}

impl<A: BitViewSized + Clone + Debug, const N: usize> Cell<A, N> {
    /// Takes a BitArray where each 1 represents a state the cell cannot be in
    /// and removes those states from the current cell
    pub fn reduce(self, reduction: CellState<A>) -> Option<Self> {
        match self {
            Self::Unknown(state) => {
                let state = state & !reduction;
                let possibilities = state
                    .iter()
                    .by_vals()
                    .enumerate()
                    .take(N)
                    .filter(|&(_, x)| x)
                    .map(|(n, _)| n)
                    .collect::<Vec<usize>>();

                match possibilities.split_first() {
                    Some((&h, &[])) => Some(Self::Reduced(state, h)),
                    Some(_) => Some(Self::Unknown(state)),
                    None => None,
                }
            }
            cell => Some(cell),
        }
    }

    /// Randomly selects a possible state
    pub fn observe(self, weights: Weights, rng: &mut StdRng) -> Result<Self, WeightedError> {
        match self {
            Self::Unknown(state) => state
                .iter_ones()
                .collect::<Vec<usize>>()
                .choose_weighted(rng, weights)
                .map(ToOwned::to_owned)
                .map(Self::reduced),
            // Self::Reduced(state, n) => Ok(Self::Collapsed(state, n)),
            // Self::Collapsed(_, _) => Ok(self),
            cell => Ok(cell),
        }
    }

    /// Updates the result for an fully collapsed cell
    pub fn collapse(self) -> Self {
        match self {
            Self::Reduced(state, n) => Self::Collapsed(state, n),
            _ => self,
        }
    }

    /// The number of possible states in the cell's superposition
    pub fn entropy(&self) -> usize {
        match self {
            Self::Unknown(state) => state.count_ones(),
            Self::Reduced(_, _) => 1,
            Self::Collapsed(_, _) => 0,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown(_) => true,
            _ => false,
        }
    }

    pub fn is_reduced(&self) -> bool {
        match self {
            Self::Reduced(_, _) => true,
            _ => false,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        match self {
            Self::Collapsed(_, _) => true,
            _ => false,
        }
    }

    pub fn unknown(state: CellState<A>) -> Self {
        Self::Unknown(state)
    }

    pub fn reduced(n: usize) -> Self {
        Self::Reduced(
            {
                let mut bits = BitArray::ZERO;
                bits.set(n, true);
                bits
            },
            n,
        )
    }

    pub fn collapsed(n: usize) -> Self {
        Self::Collapsed(
            {
                let mut bits = BitArray::ZERO;
                bits.set(n, true);
                bits
            },
            n,
        )
    }

    /// The result of the cell
    pub fn value(&self) -> Option<usize> {
        match self {
            Self::Unknown(_) => None,
            Self::Reduced(_, n) => Some(*n),
            Self::Collapsed(_, n) => Some(*n),
        }
    }

    /// The state of the cell
    pub fn state(&self) -> CellState<A> {
        match self {
            Self::Unknown(state) => state.clone(),
            Self::Reduced(state, _) => state.clone(),
            Self::Collapsed(state, _) => state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, SeedableRng};

    use super::*;

    const STATES: usize = 3;
    type Storage = u16;
    type State = BitArray<Storage, Lsb0>;
    type TestCell = Cell<Storage, STATES>;

    fn uniform(_: &usize) -> usize {
        1
    }

    #[test]
    /// Reduce a cell to no states
    fn reduce_to_none() {
        let reduction = TestCell::default().state();
        let actual = TestCell::default().reduce(reduction);
        assert!(actual.is_none())
    }

    #[test]
    /// Reduce a cell to many states
    fn reduce_to_many() {
        let reduction = TestCell::reduced(STATES - 1).state();
        let actual = TestCell::default().reduce(reduction).unwrap();
        let expected = {
            let mut bits = State::ZERO;
            for i in 0..(STATES - 1) {
                bits.set(i, true);
            }
            bits
        };
        assert!(
            actual.state() == expected,
            "Actual: {:?} Expected: {:?}",
            actual,
            expected
        );
        assert!(actual.is_unknown());
    }

    #[test]
    /// Reduce a cell to one states
    fn reduce_to_one() {
        let reduction = {
            let mut bits = State::ZERO;
            for i in 0..(STATES - 1) {
                bits.set(i, true);
            }
            bits
        };
        let actual = TestCell::default().reduce(reduction).unwrap();
        let expected = TestCell::reduced(STATES - 1).state();
        assert!(
            actual.state() == expected,
            "Actual: {:?}, Expected: {:?}",
            actual,
            expected
        );
        assert!(actual.is_reduced());
    }

    #[test]
    fn observe_empty_state() {
        let actual = TestCell::Unknown(State::ZERO)
            .observe(uniform, &mut StdRng::from_rng(thread_rng()).unwrap())
            .err();
        let expected = Result::<TestCell, WeightedError>::Err(WeightedError::NoItem).err();
        assert!(
            actual == expected,
            "Actual: {:?}, Expected: {:?}",
            actual,
            expected
        );
    }

    #[test]
    fn observe_random_state() {
        let actual = TestCell::default()
            .observe(uniform, &mut StdRng::from_rng(thread_rng()).unwrap())
            .unwrap()
            .value()
            .unwrap();
        let expected = (0..STATES).map(|i| i).collect::<Vec<usize>>();
        assert!(
            expected.contains(&actual),
            "Actual: {:?}, Expected: {:?}",
            actual,
            expected
        );
    }
}
