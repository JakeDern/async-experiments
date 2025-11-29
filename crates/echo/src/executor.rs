use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake};

use crate::{EXECUTOR, channel};

type TaskId = usize;

pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: Future<Output = T> + 'static,
    T: 'static,
{
    local_executor(move |e| e.spawn(fut))
}

pub fn make_progress() {
    local_executor(|e| e.run())
}

fn local_executor<F, T>(f: F) -> T
where
    F: FnOnce(&mut Executor) -> T,
{
    EXECUTOR.with_borrow_mut(|executor| {
        let executor = executor.as_mut().expect("Executor not initialized");
        return f(executor);
    })
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Executor {
    ready_tasks: Rc<RefCell<VecDeque<TaskId>>>,
    tasks: HashMap<TaskId, Task>,
    current_id: TaskId,
}

pub struct JoinHandle<T> {
    task_id: TaskId,
    rx: channel::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, channel::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx)
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            ready_tasks: Rc::new(RefCell::new(VecDeque::new())),
            tasks: HashMap::new(),
            current_id: 0,
        }
    }

    pub fn spawn<F, T>(&mut self, fut: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        let id = self.current_id;
        self.current_id += 1;

        let (tx, rx) = channel::oneshot::<T>();
        println!("Creating task for id: {}", id);
        let task = Task {
            id,
            future: Box::pin(async move {
                println!("Running task for id: {}", id);
                tx.send(fut.await).unwrap();
            }),
        };
        self.tasks.insert(id, task);
        self.ready_tasks.borrow_mut().push_back(id);

        JoinHandle { rx, task_id: id }
    }

    fn block_on<F, T>(&mut self, fut: F) -> Result<T, String>
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        let handle = self.spawn(fut);
        let id = handle.task_id;
        loop {
            self.run();
            if self.tasks.get(&id).is_none() {
                match handle.rx.try_recv() {
                    Some(value) => return Ok(value),
                    None => return Err(format!("Future did not resolve to a value")),
                }
            }
        }
    }

    fn run(&mut self) {
        loop {
            // We cannot hold a borrow to the ready tasks while also polling,
            // as there is a risk that the poll will cause a wake that modifies
            // the ready tasks. So we pop the task ID first, drop the borrow,
            // and then poll.
            let task_id = match self.ready_tasks.borrow_mut().pop_front() {
                Some(task_id) => task_id,
                None => break,
            };

            let Some(task) = self.tasks.get_mut(&task_id) else {
                panic!("Task in ready set, but not tracked in map.");
            };

            let waker = Arc::new(Waker {
                task_id,
                ready_tasks: self.ready_tasks.clone(),
            })
            .into();

            let mut context = Context::from_waker(&waker);
            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {
                    self.tasks.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}

pub struct Waker {
    task_id: TaskId,
    ready_tasks: Rc<RefCell<VecDeque<TaskId>>>,
}

// We're never going to send a Waker across threads, so this is safe.
unsafe impl Send for Waker {}
unsafe impl Sync for Waker {}

impl Wake for Waker {
    fn wake(self: Arc<Self>) {
        println!("Waking task {}", self.task_id);
        let mut ready = self.ready_tasks.borrow_mut();
        ready.push_back(self.task_id);
    }
}

impl Clone for Waker {
    fn clone(&self) -> Self {
        Self {
            task_id: self.task_id,
            ready_tasks: Rc::clone(&self.ready_tasks),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_spawn() {
        let mut executor = Executor::new();
        executor.spawn(async { 1 + 2 });
    }

    #[test]
    fn test_block_on() {
        let mut executor = Executor::new();
        let (tx, rx) = channel::oneshot::<i32>();
        tx.send(42).unwrap();
        let result = executor.block_on(async move {
            let res = rx.await.unwrap();
            println!("Received: {}", res);
            res
        });

        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_send_from_fut() {
        let mut executor = Executor::new();
        let (tx, rx) = channel::oneshot::<i32>();
        executor.spawn(async move {
            println!("Sending 42");
            tx.send(42).unwrap();
        });
        let result = executor.block_on(async move {
            let res = rx.await.unwrap();
            println!("Received: {}", res);
            res
        });

        assert_eq!(result.unwrap(), 42);
    }
}
