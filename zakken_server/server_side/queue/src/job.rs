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
    pub sender: Option<String>,
    pub reply: oneshot::Sender<JobReply>,
}

pub struct JobReply {
    pub bytes: Vec<u8>,
    pub just_identified: Option<String>,
}
