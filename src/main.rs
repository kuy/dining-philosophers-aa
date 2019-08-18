#![feature(async_await)]

use async_std::task;
use async_std::sync::Mutex;
use std::time::Duration;
use std::sync::Arc;

struct Philosopher {
    id: usize,
    left: Arc<Mutex<Chopstick>>,
    right: Arc<Mutex<Chopstick>>,
}

impl Philosopher {
    fn new(id: usize, left: Arc<Mutex<Chopstick>>, right: Arc<Mutex<Chopstick>>) -> Self {
        Philosopher { id, left, right }
    }

    async fn think(&self) {
        println!("Philosopher {}: Thinking...", self.id);
        task::sleep(Duration::from_secs(2)).await;
    }

    async fn eat(&self) {
        let mut left_cs = match self.left.try_lock() {
            Some(chopstick) => chopstick,
            None => {
                println!("Philosopher {}: OOPS! Left chopstick not available. Waiting...", self.id);
                self.left.lock().await
            }
        };
        left_cs.get(self);

        let mut right_cs = match self.right.try_lock() {
            Some(chopstick) => chopstick,
            None => {
                println!("Philosopher {}: OOPS! Right chopstick not available. Waiting...", self.id);
                self.right.lock().await
            }
        };
        right_cs.get(self);

        println!("Philosopher {}: Eating...", self.id);
        task::sleep(Duration::from_secs(4)).await;

        right_cs.put(self);
        left_cs.put(self);
    }
}

struct Chopstick {
    id: usize,
    by: Option<usize>,
}

impl Chopstick {
    fn new(n: usize) -> Self {
        Chopstick {
            id: n,
            by: None,
        }
    }

    fn get(&mut self, p: &Philosopher) {
        if self.by != None {
            panic!(); // Invalid state. Mutex is not working?
        }

        println!("Chopstick {}: Picked up by Philosopher {}", self.id, p.id);
        self.by = Some(p.id);
    }

    fn put(&mut self, p: &Philosopher) {
        if self.by != Some(p.id) {
            panic!(); // Invalid state. Mutex is not working?
        }

        println!("Chopstick {}: Released by Philosopher {}", self.id, p.id);
        self.by = None;
    }
}

fn deploy(n: usize, for_left: Arc<Mutex<Chopstick>>, for_right: Arc<Mutex<Chopstick>>) -> task::JoinHandle<()> {
    task::spawn(async move {
        let ph = Philosopher::new(n, for_left, for_right);
        loop {
            ph.think().await;
            ph.eat().await;
        }
    })
}

fn main() {
    let cs1 = Arc::new(Mutex::new(Chopstick::new(1)));
    let cs2 = Arc::new(Mutex::new(Chopstick::new(2)));
    let cs3 = Arc::new(Mutex::new(Chopstick::new(3)));
    let cs4 = Arc::new(Mutex::new(Chopstick::new(4)));
    let cs5 = Arc::new(Mutex::new(Chopstick::new(5)));

    let mut tasks = vec![];
    tasks.push(deploy(1, cs1.clone(), cs2.clone()));
    tasks.push(deploy(2, cs2.clone(), cs3.clone()));
    tasks.push(deploy(3, cs3.clone(), cs4.clone()));
    tasks.push(deploy(4, cs4.clone(), cs5.clone()));
    tasks.push(deploy(5, cs5.clone(), cs1.clone()));

    task::block_on(async {
        for task in tasks {
            task.await;
        }
    });
}
