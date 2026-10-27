use server_protocol::Request;
use tokio::sync::oneshot;

pub enum Priority {
    Important,
    Normal,
}

pub struct Job {
    pub message_id: usize,
    pub request: Request,
    pub payload: Vec<u8>,
    pub socket_id: u64,
    pub reply: oneshot::Sender<Vec<u8>>,
}