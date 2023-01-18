use crate::child_process::{self, ChildProcess};
use crate::error::Error;
use crate::result::Result;
// use crate::task::Task;
// use futures::Future;
use node_sys::*;
use std::path::PathBuf;
// use std::pin::Pin;
use futures::{join, select, FutureExt};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use workflow_core::channel::{Channel, Receiver};
use workflow_core::task::*;
use workflow_log::*;
use workflow_wasm::callback::*;

pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub none: bool,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Version {
        Version {
            major,
            minor,
            patch,
            none: false,
        }
    }

    pub fn none() -> Version {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            none: true,
        }
    }
}

struct Inner {
    argv: Mutex<Vec<String>>,
    cwd: Mutex<Option<PathBuf>>,
    folder: Mutex<Option<PathBuf>>,
    running: AtomicBool,
    restart: AtomicBool,
    delay: AtomicU64,
    // pid: AtomicU64,
    stdout: Channel<String>,
    exit: Channel<(u32, String)>,
    proc: Arc<Mutex<Option<ChildProcess>>>,
    callbacks: CallbackMap,
}

impl Inner {
    pub fn new(argv: &[&str]) -> Inner {
        let argv = argv.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        Inner {
            // proc,
            argv: Mutex::new(argv),
            cwd: Mutex::new(None),
            folder: Mutex::new(None),
            running: AtomicBool::new(false),
            restart: AtomicBool::new(false),
            delay: AtomicU64::new(0),
            stdout: Channel::unbounded(),
            exit: Channel::oneshot(),
            proc: Arc::new(Mutex::new(None)),
            callbacks: CallbackMap::new(),
            // monitor: Mutex::new(None),
        }
    }

    fn proc(&self) -> String {
        self.argv.lock().unwrap().get(0).unwrap().clone()
    }

    fn args(&self) -> Vec<String> {
        self.argv.lock().unwrap()[1..].to_vec()
    }

    fn cwd(&self) -> Option<PathBuf> {
        self.cwd.lock().unwrap().clone()
    }

    fn folder(&self) -> Option<PathBuf> {
        self.folder.lock().unwrap().clone()
    }

    pub async fn run(&self, stop: Receiver<()>) -> Result<()> {
        loop {

            log_info!("loop...");

            if self.running.load(Ordering::SeqCst) {
                return Err(Error::AlreadyRunning);
            }

            let cp = {
                let proc = self.proc();
                let args = &self.args();

                log_info!("proc: {:?}",proc);
                log_info!("args: {:?}",args);


                let args: child_process::SpawnArgs = args.as_slice().into();
                let options = child_process::SpawnOptions::new();
                if let Some(cwd) = &self.cwd() {
                    log_info!("cwd: {:?}", cwd);

                    options.cwd(cwd.as_os_str().to_str().expect(&format!(
                        "Process::exec_with_args(): invalid path: {}",
                        cwd.display()
                    )));
                }

                log_info!("spawn...");
                child_process::spawn_with_args_and_options(&proc, &args, &options)
            };

            log_info!("cp ready");

            let exit = self.exit.sender.clone();
            let close = callback!(move |code: u32, signal: String| {
                log_info!("close callback()...");

                exit.try_send((code, signal))
                    .expect("unable to send close notification");
            });
            cp.on("close", close.as_ref());
            self.callbacks.retain(close)?;

            let stdout = self.stdout.sender.clone();
            let data = callback!(move |data: buffer::Buffer| {
                log_info!("data callback()... {:?}", data);
                stdout
                    .try_send(String::from(data.to_string(None, None, None)))
                    .unwrap();
                log_info!("data callback done");
            });
            cp.stdout().on("data", data.as_ref());
            self.callbacks.retain(data)?;
            // }
            let r_exit = self.exit.receiver.recv();
            let r_stop = stop.recv();
            // join!(r_exit, r_stop).await?;
        log_info!("select!...");
sleep(Duration::from_millis(5000)).await;
            select! {
                v = r_exit.fuse() => {

                    log_info!("exit: {:?}",v);

                    if !self.restart.load(Ordering::SeqCst) {
                        log_info!("terminating (no restart)");
                        break;
                    } else {
                        let delay = self.delay.load(Ordering::SeqCst);
                        log_info!("restarting in {}",delay);
                        sleep(Duration::from_millis(delay)).await;
                    }

                },
                v = r_stop.fuse() => {
                    log_info!("stop: {:?}",v);

                }
            }
        }
        log_info!("loop done...");

        self.callbacks.clear();
        // TODO - WAIT ON STOP OR EXIT

        // self.monitor.run(())?;
        // self.exit
        //     .recv()
        //     .await
        //     .expect("error receiving close notification");

        // let s = self.stdout.iter().collect::<Vec<_>>().join("");

        // let mut s = String::new();
        // for _ in 0..self.stdout.len() {
        //     s.push_str(&self.stdout.try_recv()?);
        // }

        Ok(())
    }
}

pub struct Process {
    // proc: String,
    // monitor1 : Arc<GenericTask>,
    inner: Arc<Inner>,
    task: Arc<Task<Arc<Inner>, Result<()>>>, // monitor: Option<Arc<Task<(),()>>>
}

impl Process {
    pub fn new(
        // proc: String,
        args: &[&str],
    ) -> Process {
        let inner = Arc::new(Inner::new(args));

        let task = task!(|inner: Arc<Inner>, stop| async move { inner.run(stop).await });
log_info!("creating process");
        Process {
            inner,
            task: Arc::new(task),
        }
    }

    pub async fn exec_with_args(&self, args: &[&str], cwd: Option<PathBuf>) -> Result<String> {
        let proc = self.inner.proc();
        let args: child_process::SpawnArgs = args.into();
        let options = child_process::SpawnOptions::new();
        if let Some(cwd) = cwd {
            options.cwd(cwd.as_os_str().to_str().expect(&format!(
                "Process::exec_with_args(): invalid path: {}",
                cwd.display()
            )));
        }

        let cp = child_process::spawn_with_args_and_options(&proc, &args, &options);
        let exit = self.inner.exit.sender.clone();
        let close = callback!(move |code: u32, signal: String| {
            exit.try_send((code, signal))
                .expect("unable to send close notification");
        });
        cp.on("close", close.as_ref());

        let stdout = self.inner.stdout.sender.clone();
        let data = callback!(move |data: buffer::Buffer| {
            stdout.try_send(String::from(data.to_string(None, None, None)));
        });
        cp.stdout().on("data", data.as_ref());

        self.inner
            .exit
            .recv()
            .await
            .expect("error receiving close notification");

        // let s = self.stdout.iter().collect::<Vec<_>>().join("");

        let mut s = String::new();
        for _ in 0..self.inner.stdout.len() {
            s.push_str(&self.inner.stdout.try_recv()?);
        }
        Ok(s)
    }

    pub async fn get_version(&self) -> Result<Version> {
        let text = self.exec_with_args(["--version"].as_slice(), None).await?;
        let v = text
            .split(".")
            .map(|v| v.parse::<u64>())
            .flatten()
            .collect::<Vec<_>>();

        if v.len() != 3 {
            return Ok(Version::none());
        }

        Ok(Version::new(v[0], v[1], v[2]))
    }

    pub fn run(&self) -> Result<()> {
        log_info!("run...");
        self.task.run(self.inner.clone())?;
        log_info!("run done...");
        Ok(())
    }

    // async fn monitor(&self) {

    // }
}
