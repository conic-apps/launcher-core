//! The install task manager

/// Execute the corresponding function when the installation event occurs
///
/// please use `TaskEventListeners::new()` to create a new instance, and use
/// `TaskEventListeners::on_start()` `EventListeners::on_progress()`
/// `TaskEventListeners::on_succeed()` `EventListeners::on_failed()`
/// to register the event
///
/// # Examples
///
/// basic usage:
///
/// ```
/// use mgl_core::core::task::TaskEventListeners;
///
/// let listeners = TaskEventListeners::new()
///     .on_start(Box::new(|| {
///         println!("task start");
///     }))
///     .on_progress(Box::new(|completed, total, step| {
///         println!("progress: {completed}/{total}  step: {step}");
///     }));
/// ```
pub struct TaskEventListeners {
    // todo: 改成 Vec<Box<dyn Fn()>>，以允许执行多个异步
    on_start: Box<dyn Fn()>,
    on_progress: Box<dyn Fn(usize, usize, usize)>,
    on_succeed: Box<dyn Fn()>,
    on_failed: Box<dyn Fn()>,
}

impl TaskEventListeners {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            on_start: Box::new(|| {}),
            on_progress: Box::new(|_completed, _total, _step| {}),
            on_succeed: Box::new(|| {}),
            on_failed: Box::new(|| {}),
        }
    }
    /// Register the start event listener, when the task start, the event will be triggered
    pub fn on_start(self, on_start: Box<dyn Fn()>) -> Self {
        Self {
            on_start,
            ..self
        }
    }
    /// Register the progress event listener, when the task progress, the event will be triggered
    pub fn on_progress(self, on_progress: Box<dyn Fn(usize, usize, usize)>) -> Self {
        Self {
            on_progress,
            ..self
        }
    }
    /// Register the succeed event listener, when the task succeed, the event will be triggered
    pub fn on_succeed(self, on_succeed: Box<dyn Fn()>) -> Self {
        Self {
            on_succeed,
            ..self
        }
    }
    /// Register the failed event listener, when the task failed, the event will be triggered
    pub fn on_failed(self, on_failed: Box<dyn Fn()>) -> Self {
        Self {
            on_failed,
            ..self
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

/// Execute the corresponding function when the installation event occurs
///
/// please use `ProcessEventListeners::new()` to create a new instance, and use
/// `ProcessEventListeners::on_stdout()` `ProcessEventListeners::on_stderr()`
/// `ProcessEventListeners::on_exit()`
pub struct ProcessEventListeners {
    on_stdout: Box<dyn Fn(&str)>,
    /// It's not actually used
    ///
    /// todo: Supports monitoring of stderr
    on_stderr: Box<dyn Fn(&str)>,
    /// The exit code is not actually checked
    ///
    /// todo: check exit code
    on_exit: Box<dyn Fn(usize)>,
}

impl ProcessEventListeners {
    pub fn new() -> Self {
        Self {
            on_stdout: Box::new(|log| println!("{}", log)),
            on_stderr: Box::new(|log| println!("{}", log)),
            on_exit: Box::new(
                |exit_code| println!("process exited with exit_code: {exit_code}")
            ),
        }
    }
    /// Register the stdout event listener, when the stdout occurs, the event will be triggered
    pub fn on_stdout(self, on_stdout: Box<dyn Fn(&str)>) -> Self {
        Self {
            on_stdout,
            ..self
        }
    }
    /// Register the stderr event listener, when the stderr occurs, the event will be triggered
    pub fn on_stderr(self, on_stderr: Box<dyn Fn(&str)>) -> Self {
        Self {
            on_stderr,
            ..self
        }
    }
    /// Register the exit event listener, when the process end, the event will be triggered
    pub fn on_exit(self, on_exit: Box<dyn Fn(usize)>) -> Self {
        Self {
            on_exit,
            ..self
        }
    }
    pub(crate) fn stdout(&self, log: &str) {(self.on_stdout)(log);}
    pub(crate) fn stderr(&self, log: &str) {(self.on_stderr)(log);}
    pub(crate) fn exit(&self, exit_code: usize) {(self.on_exit)(exit_code);}
}

