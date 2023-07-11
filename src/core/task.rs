/*
 * Magical Launcher Core
 * Copyright (C) 2023 Broken-Deer <old_driver__@outlook.com> and contributors
 *
 * This program is free software, you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! The install task manager
//!
//! # Example
//!
//! Create listeners, then use them to monitor task progress
//!
//! ```
//! use mgl_core::core::folder::MinecraftLocation;
//! use mgl_core::core::task::TaskEventListeners;
//! use mgl_core::install::install;
//!  async fn fn_name() {
//!     let listeners = TaskEventListeners::default().on_progress(Box::new(|completed, total, step| {
//!         println!("progress: {completed}/{total}; step: {step}")
//!     }));
//!     install("1.19.4", MinecraftLocation::new(".minecraft"), listeners).await.unwrap();
//! }
//! ```

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
/// let listeners = TaskEventListeners::default()
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

impl Default for TaskEventListeners {
    fn default() -> Self {
        Self {
            on_start: Box::new(|| println!("Task is startting")),
            on_progress: Box::new(|completed, total, step| {
                println!("progress: {completed}/{total}, step: {step}")
            }),
            on_succeed: Box::new(|| println!("Done!")),
            on_failed: Box::new(|| println!("Error!")),
        }
    }
}

impl TaskEventListeners {
    /// Register the start event listener, when the task start, the event will be triggered
    pub fn on_start(self, on_start: Box<dyn Fn()>) -> Self {
        Self { on_start, ..self }
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
        Self { on_succeed, ..self }
    }
    /// Register the failed event listener, when the task failed, the event will be triggered
    pub fn on_failed(self, on_failed: Box<dyn Fn()>) -> Self {
        Self { on_failed, ..self }
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
