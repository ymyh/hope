use std::{marker::PhantomData, sync::{Arc, Mutex}};

use crossbeam::sync::WaitGroup;

use crate::{ThreadPool, joinHandle::JoinHandle};

pub struct Scope<'scope, 'env: 'scope>
{
    pub(crate) all_wg: WaitGroup,

    pool: &'env mut ThreadPool,
    env: PhantomData<&'env ()>,
    scope: PhantomData<&'scope ()>,
}

impl<'scope, 'env> Scope<'scope, 'env>
{
    pub fn new(pool: &'env mut ThreadPool) -> Self
    {
        Self {
            all_wg: WaitGroup::new(),

            pool,
            env: PhantomData,
            scope: PhantomData,
        }
    }

    pub fn spawn<F, T>(&'scope self, f: F) -> JoinHandle<'env, T>
        where F: FnOnce() -> T,
        F: Send + 'scope,
        T: Send + 'static
    {
        let f: Box<dyn FnOnce() -> T + Send + 'scope> = Box::new(f);
        let f: Box<dyn FnOnce() -> T + Send> = unsafe { std::mem::transmute(f) };

        let result = Arc::new(Mutex::new(None));
        let result2 = Arc::clone(&result);

        let wg = WaitGroup::new();
        let wg2 = wg.clone();

        let all_wg = self.all_wg.clone();

        let b: Box<dyn FnOnce() + Send> = Box::new(move ||
        {
            let res = f();
            *result2.lock().unwrap() = Some(res);

            drop(all_wg);
            drop(wg2);
        });

        self.pool.dispatch(b);

        JoinHandle::new(result, wg)
    }
}