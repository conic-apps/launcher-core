// use std::println;

// use futures::Future;

// enum TaskState {
//     Idle,
//     Running,
//     Cancelled,
//     Paused,
//     Succeed,
//     Failed,
// }

// pub struct Task<F>
// where
//     F: Future<Output = ()> + Send + 'static,
// {
//     name: String,
//     progress: usize,
//     total: usize,
//     path: String,
//     executor: F,
//     state: TaskState,
// }

// impl<F> Task<F>
// where
//     F: Future<Output = ()> + Send + 'static,
// {
//     pub fn new(name: &str, executor: F) -> Task<F> {
//         Task {
//             name: name.to_string(),
//             progress: 0,
//             total: 0,
//             state: TaskState::Idle,
//             path: "".to_string(),
//             executor,
//         }
//     }
//     pub async fn cancel() {}
// }

// async fn a(task: Task) {
//     println!("114514")
// }

// #[tokio::test]
// async fn test() {
//     let task = Task::new("114514", a());
//     task.executor.await;
// }
