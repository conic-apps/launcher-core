use futures::Future;

#[derive(Debug, Clone)]
pub struct Progress {
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    Running,
    Cancelled,
    Paused,
    Succeed,
    Failed,
}

pub struct EventListeners {
    // todo: 改成 Vec<Box<dyn Fn()>>，以允许执行多个异步
    pub on_start: Box<dyn Fn()>,
    pub on_progress: Box<dyn Fn(usize, usize)>,
    pub on_succeed: Box<dyn Fn()>,
    pub on_failed: Box<dyn Fn()>,
}

impl EventListeners {
    pub fn new() -> Self {
        Self {
            on_start: Box::new(|| {}),
            on_progress: Box::new(|_completed, _total| {}),
            on_succeed: Box::new(|| {}),
            on_failed: Box::new(|| {}),
        }
    }
    pub fn on_start(self, on_start: Box<dyn Fn()>) -> Self {
        Self {
            on_start,
            on_progress: self.on_progress,
            on_succeed: self.on_succeed,
            on_failed: self.on_failed,
        }
    }
    pub fn on_progress(self, on_progress: Box<dyn Fn(usize, usize)>) -> Self {
        Self {
            on_start: self.on_start,
            on_progress,
            on_succeed: self.on_succeed,
            on_failed: self.on_failed,
        }
    }
    pub fn on_succeed(self, on_succeed: Box<dyn Fn()>) -> Self {
        Self {
            on_start: self.on_start,
            on_progress: self.on_progress,
            on_succeed,
            on_failed: self.on_failed,
        }
    }
    pub fn on_failed(self, on_failed: Box<dyn Fn()>) -> Self {
        Self {
            on_start: self.on_start,
            on_progress: self.on_progress,
            on_succeed: self.on_succeed,
            on_failed,
        }
    }
}

pub struct Task<T>
where
    T: Future<Output = ()>,
{
    // pub task: Box<dyn Future<Output = ()>>,
    pub task: T,
    pub state: State,
    // on_progress: Box<dyn Fn(Progress)>,
    // todo: on failed and other
}

// impl<T> Task<T>
// {
//     // type Output = ();
//     pub fn new(task: Box<dyn Fn()>) -> Self {
//         Task::create(task, Box::new(|_| ()))
//     }
//     fn create(task: Box<dyn Fn()>, on_progress: Box<dyn Fn(Progress)>) -> Self {
//         Task {
//             task,
//             // on_progress,
//             state: State::Idle,
//         }
//     }
//     pub fn on_progress() {
//         // todo
//     }
//     pub async fn start_and_wait(self) {
//         // (self.task)();
//     }
// }
