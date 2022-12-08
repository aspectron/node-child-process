use node_sys::*;
//use node_sys::{stream, EventEmitter};
//use js_sys::{Function, JsString, Number, Object, Set};
use js_sys::{Object, Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module="node:child_process")]
extern {

    pub fn spawn(cmd:&str)->ChildProcess;

    #[wasm_bindgen(js_name = spawn)]
    pub fn spawn_with_args(cmd:&str, args:&SpawnArgs)->ChildProcess;

    #[wasm_bindgen(js_name = spawn)]
    pub fn spawn_with_args_and_options(
        cmd:&str,
        args:&SpawnArgs,
        options:&SpawnOptions
    )->ChildProcess;

    #[wasm_bindgen(extends = Array)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SpawnArgs;

    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SpawnOptions;

    #[wasm_bindgen(extends = EventEmitter)]
    #[derive(Clone, Debug)]
    pub type ChildProcess;

    #[wasm_bindgen(method, getter)]
    pub fn stdout(this: &ChildProcess) -> ReadableStream;//stream::Readable;

    #[wasm_bindgen(method, getter)]
    pub fn stderr(this: &ChildProcess) -> ReadableStream;//stream::Readable;

    #[wasm_bindgen(method, getter)]
    pub fn stdin(this: &ChildProcess) -> WritableStream;//stream::Writable;

}

impl From<Vec<&str>> for SpawnArgs{
    fn from(list:Vec<&str>)->Self{

        let array = Array::new();
        let mut index = 0;
        for value in list{
            array.set(index, JsValue::from(value));
            index += 1;
        }

        #[allow(unused_mut)]
        let mut args: Self = ::wasm_bindgen::JsCast::unchecked_into(array);

        args
    }
}



impl SpawnOptions {
    /// "Construct a new `SpawnOptions`.
    ///
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(Object::new());
        ret
    }

    fn set(self, key:&str, value:JsValue) -> Self{
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from(key),
            &value,
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    pub fn cwd(self, cwd:&str)->Self{
        self.set("cwd", JsValue::from(cwd))
    }

    pub fn env(self, env:ProcessEnv)->Self{
        self.set("env", JsValue::from(env))
    }

    pub fn encoding(self, encoding:&str)->Self{
        self.set("encoding", JsValue::from(encoding))
    }

    pub fn timeout(self, timeout:u32)->Self{
        self.set("timeout", JsValue::from(timeout))
    }

    pub fn max_buffer(self, max_buffer:u32)->Self{
        self.set("maxBuffer", JsValue::from(max_buffer))
    }
}
