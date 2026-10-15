use crate::job::{Job, JobReply, Priority};
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
        sender: Option<String>,
        priority: Priority,
    ) -> Result<oneshot::Receiver<JobReply>, String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let job = Job {
            message_id,
            request,
            payload,
            sender,
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
