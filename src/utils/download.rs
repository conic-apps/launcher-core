/*
 * Amethyst Launcher Core
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

// pub async fn download_files(
//     download_tasks: Vec<Download>,
//     listeners: TaskEventListeners,
//     verify_exists: bool,
// ) -> Result<()> {
//     listeners.start();
//     listeners.progress(0, 0, 1);
//     let download_tasks: Vec<_> = download_tasks
//         .iter()
//         .filter(|download_task| {
//             match std::fs::metadata(&download_task.file) {
//                 Err(_) => {
//                     return true;
//                 }
//                 _ => {
//                     if !verify_exists {
//                         return false;
//                     }
//                 }
//             }
//             let mut file = match std::fs::File::open(&download_task.file) {
//                 Ok(file) => file,
//                 Err(_) => {
//                     return true;
//                 }
//             };
//             let file_sha1 = calculate_sha1_from_read(&mut file);
//             let sha1 = match download_task.sha1.clone() {
//                 None => return true,
//                 Some(sha1) => sha1,
//             };
//             if file_sha1 == sha1 {
//                 false
//             } else {
//                 true
//             }
//         })
//         .collect();
//
//     let total = download_tasks.len();
//     let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
//
//     let stream = futures::stream::iter(download_tasks)
//         .map(|download_task| {
//             let counter = Arc::clone(&counter);
//             async move {
//                 let result = download(download_task.clone()).await;
//                 counter.fetch_add(1, Ordering::SeqCst);
//                 result
//             }
//         })
//         .buffer_unordered(16);
//     stream
//         .for_each_concurrent(1, |_| async {
//             let completed = counter.clone().load(Ordering::SeqCst);
//             listeners.progress(completed, total, 2);
//         })
//         .await;
//
//     if counter.load(Ordering::SeqCst) == total {
//         listeners.succeed();
//     } else {
//         listeners.failed();
//     }
//
//     Ok(())
// }
