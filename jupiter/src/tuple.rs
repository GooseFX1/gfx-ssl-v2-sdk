use std::{
    mem::zeroed,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy)]
pub struct Tuple<const N: usize, T>([T; N]);

impl<const N: usize, T> Default for Tuple<N, T>
where
    T: Default,
{
    fn default() -> Self {
        let mut this: [T; N] = unsafe { zeroed() };
        for e in &mut this {
            *e = Default::default();
        }
        Tuple(this)
    }
}

impl<const N: usize, T> Tuple<N, T> {
    pub fn new(value: [T; N]) -> Self {
        Self(value)
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        for e in &self.0 {
            f(e)
        }
    }

    pub fn all<F>(&self, mut f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        let mut b = true;
        for e in &self.0 {
            b &= f(e);
            if !b {
                return b;
            }
        }

        true
    }

    pub fn any<F>(&self, mut f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        let mut b = false;
        for e in &self.0 {
            b |= f(e);
            if b {
                return b;
            }
        }

        false
    }
}

impl<const N: usize, T> Tuple<N, T>
where
    T: Clone,
{
    pub fn to_vec(&self) -> Vec<T> {
        self.0.to_vec()
    }

    pub fn pick<const M: usize>(&self, indices: [usize; M]) -> Tuple<M, T> {
        indices.map(|i| self[i].clone()).into()
    }

    pub fn reverse(&self) -> Tuple<N, T> {
        let mut ret = self.0.clone();
        ret.reverse();
        ret.into()
    }
}

impl<T> Into<Tuple<2, T>> for (T, T) {
    fn into(self) -> Tuple<2, T> {
        Tuple::new([self.0, self.1])
    }
}

impl<const N: usize, T> Into<Tuple<N, T>> for [T; N] {
    fn into(self) -> Tuple<N, T> {
        Tuple::new(self)
    }
}

impl<const N: usize, T> Deref for Tuple<N, T> {
    type Target = [T; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, T> DerefMut for Tuple<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
