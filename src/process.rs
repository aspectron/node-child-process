use crate::child_process::{self, ChildProcess, KillSignal};
use crate::error::Error;
use crate::result::Result;
use futures::{select, FutureExt};
use node_sys::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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

pub struct Options {
    argv: Vec<String>,
    cwd: Option<PathBuf>,
    folder: Option<PathBuf>,
    restart: bool,
    delay: u64,
    // env : HashMap<String, String>,
}

impl Options {
    pub fn new(
        argv: &[&str],
        cwd: Option<PathBuf>,
        folder: Option<PathBuf>,
        restart: bool,
        delay: Option<u64>,
    ) -> Options {
        let argv = argv.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        Options {
            argv,
            cwd,
            folder,
            restart,
            delay: delay.unwrap_or(3000),
        }
    }
}

struct Inner {
    argv: Mutex<Vec<String>>,
    cwd: Mutex<Option<PathBuf>>,
    running: AtomicBool,
    restart: AtomicBool,
    delay: AtomicU64,
    stdout: Channel<String>,
    stderr: Channel<String>,
    exit: Channel<u32>,
    proc: Arc<Mutex<Option<ChildProcess>>>,
    callbacks: CallbackMap,
}

impl Inner {
    pub fn new(options: &Options) -> Inner {
        Inner {
            argv: Mutex::new(options.argv.clone()),
            cwd: Mutex::new(options.cwd.clone()),
            running: AtomicBool::new(false),
            restart: AtomicBool::new(options.restart),
            delay: AtomicU64::new(options.delay),
            stdout: Channel::unbounded(),
            stderr: Channel::unbounded(),
            exit: Channel::oneshot(),
            proc: Arc::new(Mutex::new(None)),
            callbacks: CallbackMap::new(),
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

    pub async fn run(&self, stop: Receiver<()>) -> Result<()> {
        loop {
            log_info!("loop...");

            if self.running.load(Ordering::SeqCst) {
                return Err(Error::AlreadyRunning);
            }

            let cp = {
                let proc = self.proc();
                let args = &self.args();

                let args: child_process::SpawnArgs = args.as_slice().into();
                let options = child_process::SpawnOptions::new();
                if let Some(cwd) = &self.cwd() {
                    options.cwd(cwd.as_os_str().to_str().unwrap_or_else(|| {
                        panic!("Process::exec_with_args(): invalid path: {}", cwd.display())
                    }));
                }

                child_process::spawn_with_args_and_options(&proc, &args, &options)
            };

            let exit = self.exit.sender.clone();
            let close = callback!(move |code: u32| {
                exit.try_send(code)
                    .expect("unable to send close notification");
            });
            cp.on("close", close.as_ref());
            self.callbacks.retain(close.clone())?;

            let stdout_tx = self.stdout.sender.clone();
            let stdout_cb = callback!(move |data: buffer::Buffer| {
                stdout_tx
                    .try_send(String::from(data.to_string(None, None, None)))
                    .unwrap();
            });
            cp.stdout().on("data", stdout_cb.as_ref());
            self.callbacks.retain(stdout_cb)?;

            let stderr_tx = self.stderr.sender.clone();
            let stderr_cb = callback!(move |data: buffer::Buffer| {
                stderr_tx
                    .try_send(String::from(data.to_string(None, None, None)))
                    .unwrap();
            });
            cp.stderr().on("data", stderr_cb.as_ref());
            self.callbacks.retain(stderr_cb)?;

            *self.proc.lock().unwrap() = Some(cp);
            self.running.store(true,Ordering::SeqCst);

            select! {
                v = self.exit.receiver.recv().fuse() => {

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
                v = stop.recv().fuse() => {
                    log_info!("stop: {:?}",v);

                }
            }
        }
        log_info!("loop done...");

        self.callbacks.clear();
        *self.proc.lock().unwrap() = None;
        self.running.store(false,Ordering::SeqCst);

        Ok(())
    }
}

#[derive(Clone)]
pub struct Process {
    inner: Arc<Inner>,
    task: Arc<Task<Arc<Inner>, ()>>,
}

impl Process {
    pub fn new(options: &Options) -> Process {
        let inner = Arc::new(Inner::new(options));

        let task = task!(|inner: Arc<Inner>, stop| async move { inner.run(stop).await; });
        log_info!("creating process");
        Process {
            inner,
            task: Arc::new(task),
        }
    }

    pub fn stdout(&self) -> Receiver<String> {
        self.inner.stdout.receiver.clone()
    }

    pub fn stderr(&self) -> Receiver<String> {
        self.inner.stderr.receiver.clone()
    }

    pub async fn exec_with_args(&self, args: &[&str], cwd: Option<PathBuf>) -> Result<String> {
        let proc = self.inner.proc();
        let args: child_process::SpawnArgs = args.into();
        let options = child_process::SpawnOptions::new();
        if let Some(cwd) = cwd {
            options.cwd(cwd.as_os_str().to_str().unwrap_or_else(|| {
                panic!("Process::exec_with_args(): invalid path: {}", cwd.display())
            }));
        }

        let cp = child_process::spawn_with_args_and_options(&proc, &args, &options);
        let exit = self.inner.exit.sender.clone();
        let close = callback!(move |code: u32| {
            exit.try_send(code)
                .expect("unable to send close notification");
        });
        cp.on("close", close.as_ref());

        let stdout_tx = self.inner.stdout.sender.clone();
        let stdout_cb = callback!(move |data: buffer::Buffer| {
            stdout_tx
                .try_send(String::from(data.to_string(None, None, None)))
                .expect("unable to send stdout data");
        });
        cp.stdout().on("data", stdout_cb.as_ref());

        self.inner
            .exit
            .recv()
            .await
            .expect("error receiving close notification");

        let mut s = String::new();
        for _ in 0..self.inner.stdout.len() {
            s.push_str(&self.inner.stdout.try_recv()?);
        }
        Ok(s)
    }

    pub async fn get_version(&self) -> Result<Version> {
        let text = self.exec_with_args(["--version"].as_slice(), None).await?;
        let v = text
            .split('.')
            .flat_map(|v| v.parse::<u64>())
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

    pub fn stop(&self) -> Result<()> {
        log_info!("running.load");
        if !self.inner.running.load(Ordering::SeqCst) {
            return Err(Error::NotRunning);
        }
        
        log_info!("running.restart store");
        self.inner.restart.store(false, Ordering::SeqCst);
        log_info!("last");
        
        if let Some(proc) = self.inner.proc.lock().unwrap().as_ref() {
            log_info!("kill");
            proc.kill_with_signal(KillSignal::Message("SIGKILL".to_string()));
        } else {
            log_info!("no proc");
            return Err(Error::ProcIsAbsent);
        }
        log_info!("stop is done");
        
        Ok(())
    }

    pub async fn join(&self) -> Result<()> {
        self.task.join().await?;
        Ok(())
    }

    pub async fn stop_and_join(&self) -> Result<()> {
        log_info!("calling stop();");
        self.stop()?;
        log_info!("calling join();");
        self.join().await?;
        Ok(())
    }
}
