use std::{marker::PhantomData, sync::{Arc, Mutex}};

use crossbeam::sync::WaitGroup;

pub struct JoinHandle<'env, T>
{
    data: Arc<Mutex<Option<T>>>,
    wg: WaitGroup,
    env: PhantomData<&'env ()>,
}

impl<'env, T> JoinHandle<'env, T>
{
    pub(crate) fn new(data: Arc<Mutex<Option<T>>>, wg: WaitGroup) -> Self
    {
        Self {
            data,
            wg,
            env: PhantomData,
        }
    }

    pub fn join(self) -> Result<T, ()>
    {
        self.wg.wait();

        match self.data.lock()
        {
            Ok(mut v) =>
            {
                Ok(v.take().unwrap())
            }

            Err(_) =>
            {
                Err(())
            }
        }
    }
}