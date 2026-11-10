use server_protocol::Request;
use tokio::sync::oneshot;

pub enum Priority {
    Important,
    Normal,
}

/// Whether this job is a message some client asked for (in which case
/// the client told us what drift version it thinks the server is on),
/// or something the socket asked on its own behalf (the internal pull,
/// which has no client and therefore no drift version to check).
pub enum WhereTheRequestCameFrom {
    FromAClient(Option<String>),
    FromTheServer,
}

pub struct Job {
    pub message_id: usize,
    pub request: Request,
    pub payload: Vec<u8>,
    pub socket_id: u64,
    pub reply: oneshot::Sender<Vec<u8>>,
    pub where_the_request_came_from: WhereTheRequestCameFrom,
}