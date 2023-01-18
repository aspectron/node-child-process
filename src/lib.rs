mod error;
mod process;
mod result;

use process::Process;
// use std::future::Future;
// use std::sync::Arc;
// use std::time::Duration;

use node_sys::*;
use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;
use workflow_log::*;
pub mod child_process;
pub use child_process::{spawn, spawn_with_args, spawn_with_args_and_options};
use workflow_wasm::callback::*;
// use workflow_core::channel::oneshot;
use workflow_core::channel::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn require(s: &str) -> JsValue;
}

#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// pub struct TaskInner {
//     pub terminate_send: Sender<()>,
//     pub terminate_complete: Receiver<()>,
//     // pub closure :
// }

#[wasm_bindgen]
pub async fn test() {
    log_info!("running rust test() fn");
    // workflow_wasm::panic::init_console_panic_hook();
    //log_info!("process.pid:{:?}", process.pid());
    //let id = process.get_uid();
    //log_info!("process.get_gid(): id:{}, {:?}", id, process.get_gid());
    //process.kill(id.try_into().unwrap());

    let proc = Process::new(&["ls","-m","-s"]);


    proc.run();


    // let args: child_process::SpawnArgs = ["-m", "-s"].as_slice().into(); // = child_process::SpawnArgs::from(&["-m", "-s"]);
    //                                                                      // let args = child_process::SpawnArgs::from(["-m", "-s"].as_slice());
    // let options = child_process::SpawnOptions::new();
    // options.cwd("../");

    // //let cp = spawn("ls");
    // //let cp = spawn_with_args("ls", &args);
    // let cp = spawn_with_args_and_options("ls", &args, &options);

    // //log_info!("spawn('ls'): {:#?}", cp);

    // // let close_callback = Closure::<dyn Fn(buffer::Buffer)>::new(move |data:buffer::Buffer|{
    // //     log_info!("close: {}", data.to_string(None, None, None));
    // // });
    // // let data_callback = Closure::<dyn Fn(buffer::Buffer)>::new(move |data:buffer::Buffer|{
    // //     log_info!("data: {}", data.to_string(None, None, None));
    // // });

    // let (sender, receiver) = oneshot();

    // // cp.on("close", close_callback.as_ref().unchecked_ref());
    // let close = callback!(move |data: buffer::Buffer| {
    //     log_info!("close: {}", data.to_string(None, None, None));
    //     sender
    //         .try_send(())
    //         .expect("unable to send close notification");
    // });
    // cp.on("close", close.as_ref());
    // // cp.stdout().on("data", data_callback.as_ref().unchecked_ref());
    // let data = callback!(move |data: buffer::Buffer| {
    //     log_info!("data: {}", data.to_string(None, None, None));
    // });
    // cp.stdout().on("data", data.as_ref());

    // receiver
    //     .recv()
    //     .await
    //     .expect("error receiving close notification");
    // // close_callback.forget();
    // // data_callback.forget();

    // //let p = require("process");
    // //log_info!("process: {:?}", p);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
