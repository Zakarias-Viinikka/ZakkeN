use crate::job::{Job, Priority};
use server_protocol::Request;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct QueueManager {
    important_tx: mpsc::UnboundedSender<Job>,
    normal_tx: mpsc::UnboundedSender<Job>,
}

impl QueueManager {
    pub fn new(
        important_tx: mpsc::UnboundedSender<Job>,
        normal_tx: mpsc::UnboundedSender<Job>,
    ) -> Self {
        Self {
            important_tx,
            normal_tx,
        }
    }

    pub fn submit(
        &self,
        message_id: usize,
        request: Request,
        payload: Vec<u8>,
        socket_id: u64,
        priority: Priority,
    ) -> Result<oneshot::Receiver<Vec<u8>>, String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let job = Job {
            message_id,
            request,
            payload,
            socket_id,
            reply: reply_tx,
        };

        let tx = match priority {
            Priority::Important => &self.important_tx,
            Priority::Normal => &self.normal_tx,
        };
        tx.send(job).map_err(|e| format!("queue is closed: {}", e))?;

        Ok(reply_rx)
    }
}