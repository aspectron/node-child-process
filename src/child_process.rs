use node_sys::*;
use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "node:child_process")]
extern "C" {

    pub fn spawn(cmd: &str) -> ChildProcess;

    #[wasm_bindgen(js_name = spawn)]
    pub fn spawn_with_args(cmd: &str, args: &SpawnArgs) -> ChildProcess;

    #[wasm_bindgen(js_name = spawn)]
    pub fn spawn_with_args_and_options(
        cmd: &str,
        args: &SpawnArgs,
        options: &SpawnOptions,
    ) -> ChildProcess;

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
    pub fn exit_code(this: &ChildProcess) -> u64; //stream::Readable;

    #[wasm_bindgen(method, getter)]
    pub fn pid(this: &ChildProcess) -> u64; //stream::Readable;

    #[wasm_bindgen(method, getter)]
    pub fn stdout(this: &ChildProcess) -> ReadableStream; //stream::Readable;

    #[wasm_bindgen(method, getter)]
    pub fn stderr(this: &ChildProcess) -> ReadableStream; //stream::Readable;

    #[wasm_bindgen(method, getter)]
    pub fn stdin(this: &ChildProcess) -> WritableStream; //stream::Writable;

    #[wasm_bindgen(method)]
    pub fn kill(this: &ChildProcess) -> bool;

    #[wasm_bindgen(method, js_name=kill)]
    fn kill_with_signal_impl(this: &ChildProcess, signal: JsValue) -> bool;
}

unsafe impl Send for ChildProcess {}
unsafe impl Sync for ChildProcess {}

pub enum KillSignal<'s> {
    None,
    SIGKILL,
    SIGTERM,
    Message(&'s str),
    Code(u32),
}

impl ChildProcess {
    pub fn kill_with_signal(self: &ChildProcess, signal: KillSignal) -> bool {
        match signal {
            KillSignal::None => self.kill(),
            KillSignal::SIGKILL => self.kill_with_signal_impl(JsValue::from("SIGKILL")),
            KillSignal::SIGTERM => self.kill_with_signal_impl(JsValue::from("SIGTERM")),
            KillSignal::Message(str) => self.kill_with_signal_impl(JsValue::from(str)),
            KillSignal::Code(code) => self.kill_with_signal_impl(JsValue::from(code)),
        }
    }
}

impl From<Vec<&str>> for SpawnArgs {
    fn from(list: Vec<&str>) -> Self {
        let array = Array::new();
        for (index, value) in list.iter().enumerate() {
            array.set(index as u32, JsValue::from(*value));
        }

        #[allow(unused_mut)]
        let mut args: Self = ::wasm_bindgen::JsCast::unchecked_into(array);
        args
    }
}

impl From<&[&str]> for SpawnArgs {
    fn from(list: &[&str]) -> Self {
        let array = Array::new();
        for (index, value) in list.iter().enumerate() {
            array.set(index as u32, JsValue::from(*value));
        }

        #[allow(unused_mut)]
        let mut args: Self = ::wasm_bindgen::JsCast::unchecked_into(array);
        args
    }
}

impl From<&[String]> for SpawnArgs {
    fn from(list: &[String]) -> Self {
        let array = Array::new();
        for (index, value) in list.iter().enumerate() {
            array.set(index as u32, JsValue::from(value));
        }

        #[allow(unused_mut)]
        let mut args: Self = ::wasm_bindgen::JsCast::unchecked_into(array);
        args
    }
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SpawnOptions {
    /// "Construct a new `SpawnOptions`.
    ///
    /// [NODEJS Documentation](https://nodejs.org/api/child_process.html#child_processspawncommand-args-options)
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(Object::new());
        ret
    }

    pub fn set(&self, key: &str, value: JsValue) -> &Self {
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from(key), &value);
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }

    pub fn cwd(&self, cwd: &str) -> &Self {
        self.set("cwd", JsValue::from(cwd))
    }

    pub fn env(&self, env: ProcessEnv) -> &Self {
        self.set("env", JsValue::from(env))
    }

    pub fn argv0(&self, argv0: &str) -> &Self {
        self.set("argv0", JsValue::from(argv0))
    }

    pub fn detached(&self, detached: bool) -> &Self {
        self.set("detached", JsValue::from(detached))
    }

    pub fn uid(&self, uid: &str) -> &Self {
        self.set("uid", JsValue::from(uid))
    }

    pub fn gid(&self, gid: &str) -> &Self {
        self.set("gid", JsValue::from(gid))
    }

    pub fn serialization(&self, serialization: &str) -> &Self {
        self.set("serialization", JsValue::from(serialization))
    }

    pub fn shell(&self, shell: bool) -> &Self {
        self.set("shell", JsValue::from(shell))
    }

    pub fn shell_str(&self, shell: &str) -> &Self {
        self.set("shell", JsValue::from(shell))
    }

    pub fn windows_verbatim_arguments(&self, args: bool) -> &Self {
        self.set("windowsVerbatimArguments", JsValue::from(args))
    }

    pub fn windows_hide(&self, windows_hide: bool) -> &Self {
        self.set("windowsHide", JsValue::from(windows_hide))
    }

    pub fn timeout(&self, timeout: u32) -> &Self {
        self.set("timeout", JsValue::from(timeout))
    }

    // TODO: AbortSignal

    pub fn kill_signal(&self, signal: u32) -> &Self {
        self.set("killSignal", JsValue::from(signal))
    }

    pub fn kill_signal_str(&self, signal: &str) -> &Self {
        self.set("killSignal", JsValue::from(signal))
    }
}
