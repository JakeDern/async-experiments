use std::collections::HashMap;
use std::pin::Pin;

use crate::channel;

type TaskId = usize;

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Executor {
    tasks: HashMap<TaskId, Task>,
    current_id: TaskId,
}

pub struct JoinHandle<T> {
    task_id: TaskId,
    rx: channel::Receiver<T>,
}

impl Executor {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            current_id: 0,
        }
    }

    fn spawn<F, T>(&mut self, fut: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        let id = self.current_id;
        self.current_id += 1;

        let (tx, rx) = channel::oneshot::<T>();
        let task = Task {
            id,
            future: Box::pin(async move {
                tx.send(fut.await).unwrap();
            }),
        };
        self.tasks.insert(id, task);

        JoinHandle { rx, task_id: id }
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
}
