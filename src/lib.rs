mod error;
mod process;
mod result;

pub mod child_process;
pub use child_process::*;

// pub struct TaskInner {
//     pub terminate_send: Sender<()>,
//     pub terminate_complete: Receiver<()>,
//     // pub closure :
// }
