use std::ops::Index;

/// instance draw的时候才比较好用
#[derive(Clone, Default)]
pub struct Attribute<R: Clone + Copy, T: Index<usize, Output = R> + Default = Vec<R>>
{
    data: T,
    idx: usize,
    forward_every_n_iter: u32,
    n: u32,
}

impl<R: Clone + Copy, T: Index<usize, Output = R> + Default> Attribute<R, T>
{
    pub fn new(data: T, forward_every_n_iter: u32) -> Self
    {
        Self {
            data,
            idx: 0,
            forward_every_n_iter,
            n: 0,
        }
    }

    pub fn reset(&mut self)
    {
        self.n = 0;
        self.idx = 0;
    }

    pub fn forward(&mut self)
    {
        self.n += 1;

        if self.n == self.forward_every_n_iter
        {
            self.idx += 1;
            self.n = 0;
        }
    }

    pub fn get(&self) -> R
    {
        self.data[self.idx]
    }
}