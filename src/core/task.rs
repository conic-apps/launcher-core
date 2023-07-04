//! The install task manager
/// Execute the corresponding function when the installation event occurs
///
/// please use `EventListeners::new()` to create a new instance, and use
/// `EventListeners::on_start()` `EventListeners::on_progress()`
/// `EventListeners::on_succeed()` `EventListeners::on_failed()`
/// to register the event
///
/// # Examples
///
/// basic usage:
///
/// ```
/// use mgl_core::core::task::EventListeners;
///
/// let listeners = EventListeners::new()
///     .on_start(Box::new(|| {
///         println!("task start");
///     }))
///     .on_progress(Box::new(|completed, total, step| {
///         println!("progress: {completed}/{total}  step: {step}");
///     }));
/// ```
pub struct EventListeners {
    // todo: 改成 Vec<Box<dyn Fn()>>，以允许执行多个异步
    on_start: Box<dyn Fn()>,
    on_progress: Box<dyn Fn(usize, usize, usize)>,
    on_succeed: Box<dyn Fn()>,
    on_failed: Box<dyn Fn()>,
}

impl EventListeners {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            on_start: Box::new(|| {}),
            on_progress: Box::new(|_completed, _total, _step| {}),
            on_succeed: Box::new(|| {}),
            on_failed: Box::new(|| {}),
        }
    }
    /// Register the start event listener, when the task start, the
    /// event will be triggered
    pub fn on_start(self, on_start: Box<dyn Fn()>) -> Self {
        Self {
            on_start,
            on_progress: self.on_progress,
            on_succeed: self.on_succeed,
            on_failed: self.on_failed,
        }
    }
    /// Register the progress event listener, when the task progress, the
    /// event will be triggered
    pub fn on_progress(self, on_progress: Box<dyn Fn(usize, usize, usize)>) -> Self {
        Self {
            on_start: self.on_start,
            on_progress,
            on_succeed: self.on_succeed,
            on_failed: self.on_failed,
        }
    }
    /// Register the succeed event listener, when the task succeed, the
    /// event will be triggered
    pub fn on_succeed(self, on_succeed: Box<dyn Fn()>) -> Self {
        Self {
            on_start: self.on_start,
            on_progress: self.on_progress,
            on_succeed,
            on_failed: self.on_failed,
        }
    }
    /// Register the failed event listener, when the task failed, the
    /// event will be triggered
    pub fn on_failed(self, on_failed: Box<dyn Fn()>) -> Self {
        Self {
            on_start: self.on_start,
            on_progress: self.on_progress,
            on_succeed: self.on_succeed,
            on_failed,
        }
    }
    pub(crate) fn start(&self) {
        (self.on_start)();
    }
    pub(crate) fn progress(&self, completed: usize, total: usize, step: usize) {
        (self.on_progress)(completed, total, step);
    }
    pub(crate) fn succeed(&self) {
        (self.on_succeed)();
    }
    pub(crate) fn failed(&self) {
        (self.on_failed)();
    }
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
