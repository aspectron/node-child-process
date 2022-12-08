use wasm_bindgen::prelude::*;
use node_sys::*;
use workflow_log::*;

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

    let p = require("process");
    log_info!("process: {:?}", p);

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
