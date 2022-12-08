use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use node_sys::*;
use workflow_log::*;
pub mod child_process;
pub use child_process::{spawn, spawn_with_args, spawn_with_args_and_options};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn require(s: &str) -> JsValue;
}
 
#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[wasm_bindgen]
pub fn test() {

    log_info!("running rust test() fn");
    //log_info!("process.pid:{:?}", process.pid());
    //let id = process.get_uid();
    //log_info!("process.get_gid(): id:{}, {:?}", id, process.get_gid());
    //process.kill(id.try_into().unwrap());


    let args = child_process::SpawnArgs::from(vec!["-m", "-s"]);
    let options = child_process::SpawnOptions::new()
        .cwd("../");

    //let cp = spawn("ls");
    //let cp = spawn_with_args("ls", &args);
    let cp = spawn_with_args_and_options("ls", &args, &options);

    //log_info!("spawn('ls'): {:#?}", cp);

    let close_callback = Closure::<dyn Fn(buffer::Buffer)>::new(move |data:buffer::Buffer|{
        log_info!("close: {}", data.to_string(None, None, None));
    });
    let data_callback = Closure::<dyn Fn(buffer::Buffer)>::new(move |data:buffer::Buffer|{
        log_info!("data: {}", data.to_string(None, None, None));
    });

    cp.on("close", close_callback.as_ref().unchecked_ref());
    cp.stdout().on("data", data_callback.as_ref().unchecked_ref());

    close_callback.forget();
    data_callback.forget();

    //let p = require("process");
    //log_info!("process: {:?}", p);

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
