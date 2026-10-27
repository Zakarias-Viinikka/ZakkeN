use super::handle::NudgeReport;
use error_stuff::ServErr;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

pub enum Command {
    RegisterSocket {
        socket_id: u64,
        sender: UnboundedSender<()>,
    },
    RemoveSocket {
        socket_id: u64,
    },
    AttachName {
        socket_id: u64,
        user_name: String,
        reply: oneshot::Sender<()>,
    },
    UserOf {
        socket_id: u64,
        reply: oneshot::Sender<Option<String>>,
    },
    NudgeSocket {
        socket_id: u64,
        reply: oneshot::Sender<Result<(), ServErr>>,
    },
    NudgeAll {
        except: Option<String>,
        reply: oneshot::Sender<NudgeReport>,
    },
    IsConnected {
        user_name: String,
        reply: oneshot::Sender<bool>,
    },
    ConnectedUserNames {
        reply: oneshot::Sender<Vec<String>>,
    },
}