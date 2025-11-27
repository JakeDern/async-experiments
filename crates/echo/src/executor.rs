use std::collections::HashMap;
use std::pin::Pin;

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
    data: Option<T>,
}

impl Executor {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            current_id: 0,
        }
    }

    fn spawn<F>(&mut self, fut: F)
    where
        F: Future + 'static,
    {
        let task = Task {
            id: self.current_id,
            future: Box::pin(async move {
                let result = fut.await;
            }),
        };
        self.current_id += 1;
    }
}
