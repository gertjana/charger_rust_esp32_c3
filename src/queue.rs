use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

pub trait Queue<T> {
    fn new() -> Self;
    fn push(&self, value: T);
    fn pop(&self) -> T;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub struct FifoQueue<T> {
    pub data: Mutex<VecDeque<T>>,
    pub cv: Condvar,
}

impl<T> Queue<T> for FifoQueue<T> {
    fn new() -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
            cv: Condvar::new(),
        }
    }

    fn push(&self, value: T) {
        let mut data = self.data.lock().unwrap();
        data.push_back(value);

        self.cv.notify_one();
    }

    fn pop(&self) -> T {
        let mut data = self.data.lock().unwrap();

        // wait for the notification if the queue is empty
        while data.is_empty() {
            data = self.cv.wait(data).unwrap();
        }

        data.pop_front().unwrap()
    }

    fn len(&self) -> usize {
        let data = self.data.lock().unwrap();
        data.len()
    }

    fn is_empty(&self) -> bool {
        let data = self.data.lock().unwrap();
        data.is_empty()
    }
}
