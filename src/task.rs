use workflow_core::channel::*;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;
// use node_sys::*;
use workflow_log::*;
use crate::result::Result;


#[derive(Clone)]
pub struct Task<F,T> 
where
    F: Future<Output = T> + 'static,
    T: 'static,
{
    terminate : Sender<()>,
    sender : Sender<()>,
    done : Receiver<()>,
    receiver : Receiver<()>,
    running : Arc<AtomicBool>,

    task_fn: Arc<Box<dyn Fn(Arc<Box<Receiver<()>>>)->F>>
}

// impl<F,T> Default for Task<F,T> 
// where
//     F: Future<Output = T> + 'static,
//     T: 'static,
// {
//     fn default() -> Self {

//     }
// }
impl<F,T> Task<F,T>
where
    F: Future<Output = T> + 'static,
    T: 'static,
{

    // pub fn new(task_fn: Box<dyn Fn(Arc<Box<Receiver<()>>>)->F>) -> Task<F,T> {
    //     let mut task = Task::default();
    //     task.set_fn(task_fn);
    //     task
    // }

    pub fn new(task_fn: Box<dyn Fn(Arc<Box<Receiver<()>>>)->F>) -> Task<F,T> {
        // let mut task = Task::default();
        // task.task_fn = Some();
        
        
        let (terminate,receiver) = oneshot();
        let (sender,done) = oneshot();

        Task {
            sender, receiver,
            terminate,
            done,
            running : Arc::new(AtomicBool::new(false)),
            task_fn : Arc::new(task_fn),
        }

    }

    // where
    // F: Future<Output = T> + 'static,
    // T: 'static,
    async fn run(self : &Arc<Self>) -> Result<()>
    {
        while self.done.len() > 0 {
            self.done.try_recv()?;
        }

        while self.receiver.len() > 0 {
            self.receiver.try_recv()?;
        }

        let this = self.clone();
        workflow_core::task::spawn(async move {
            this.running.store(true, Ordering::SeqCst);
            (this.task_fn)(Arc::new(Box::new(this.receiver.clone()))).await;
            this.running.store(false, Ordering::SeqCst);
            this.sender.send(()).await.expect("Error signaling task completion");
        });

        Ok(())
    }

    pub async fn terminate(&self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            self.terminate.send(()).await?;
        }
        Ok(())
    }

    pub async fn wait(&self) -> Result<()> {
        // self.terminate.send(()).await?;
        if self.running.load(Ordering::SeqCst) {
            self.done.recv().await?;
        }
        Ok(())
    }
}


#[wasm_bindgen]
pub async fn test_task() {


    // let task = Task::new(Box::new(FnMut(Receiver<()>) -> Result<()> {
    let task = Arc::new(Task::new(Box::new(|receiver:Arc<Box<Receiver<()>>>| async move {
        for i in 0..10 {
            if receiver.try_recv().is_ok() {
                log_info!("exiting task...");
                break;
            }
            log_info!("t: {}",i);
            workflow_core::task::sleep(Duration::from_millis(500)).await;
        }

    })));

    task.run().await.ok();

    for i in 0..5 {
        log_info!("m: {}",i);
        workflow_core::task::sleep(Duration::from_millis(500)).await;
    }

    task.wait().await.ok();
    task.terminate().await.ok();

    task.run().await.ok();

    for i in 0..5 {
        log_info!("m: {}",i);
        workflow_core::task::sleep(Duration::from_millis(500)).await;
    }

    task.terminate().await.ok();
    task.wait().await.ok();

    log_info!("done");

    // task.done.recv().await.expect("Error receiving task completion");


}