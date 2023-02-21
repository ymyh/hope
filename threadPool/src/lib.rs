#![allow(non_snake_case)]

pub mod joinHandle;
pub mod scope;

use std::{cell::Cell, thread::spawn};

use crossbeam::channel::{Sender, self};
use scope::Scope;

pub struct ThreadPool
{
    count: u32,
    task_senders: Cell<Vec<Sender<Box<dyn FnOnce() + Send>>>>,
    idx: Cell<usize>,
}

impl<'env> ThreadPool
{
    pub fn new(count: u32) -> Self
    {
        let this = Self {
            count,
            idx: Cell::new(0 as usize),
            task_senders: Cell::new(Vec::new()),
        };

        for _ in 0..count
        {
            this.spawn_thread();
        }

        this
    }

    pub fn thread_count(&self) -> u32
    {
        self.count
    }

    pub fn scope<F, T>(&'env mut self, f: F) -> T
    where
        F: for<'scope> FnOnce(&Scope<'scope, 'env>) -> T
    {
        let scope = Scope::new(self);
        let r = f(&scope);

        scope.all_wg.wait();

        r
    }

    pub(crate) fn dispatch(&'env self, func: Box<dyn FnOnce() + Send>)
    {
        let mut idx = self.idx.get();
        let mut task_senders = self.task_senders.take();

        if idx == task_senders.len()
        {
            self.idx.set(0);
            idx = 0;
        }

        if let Err(e) = task_senders[idx].send(func)
        {
            task_senders.remove(idx);
            self.task_senders.replace(task_senders);

            self.spawn_thread();
            self.dispatch(e.0);
        }
        else
        {
            self.task_senders.replace(task_senders);
        }
        
        self.idx.set(idx + 1);
    }

    pub(crate) fn spawn_thread(&self)
    {
        let (sender, receiver) = channel::unbounded();
        let mut task_senders = self.task_senders.take();

        task_senders.push(sender);

        spawn(move ||
        {
            while let Ok(func) = receiver.recv()
            {
                func();
            }
        });

        self.task_senders.replace(task_senders);
    }
}