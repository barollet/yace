use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct ConstIndexed<T, const N: usize>(pub [T; N]);

pub type ColorIndexed<T> = ConstIndexed<T, 2>;
pub type PieceIndexed<T> = ConstIndexed<T, 6>;

impl<T, const N: usize> ConstIndexed<T, N> where T: Default + Copy {
    pub fn new() -> Self {
        Self([T::default(); N])
    }
}

impl<T, const N: usize> Default for ConstIndexed<T, N> where T: Default + Copy {    
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize, I: Into<usize>> Index<I> for ConstIndexed<T, N> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index.into()]
    }
}

impl<T, const N: usize, I: Into<usize>> IndexMut<I> for ConstIndexed<T, N> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.0[index.into()]
    }
}