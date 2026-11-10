use super::handle::NotifyReport;
use super::telephone_command::TelephoneCommand;
use error_stuff::ServErr;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

pub enum Command {
    RegisterSocket {
        socket_id: u64,
        sender: UnboundedSender<TelephoneCommand>,
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
    NotifySocket {
        socket_id: u64,
        reply: oneshot::Sender<Result<(), ServErr>>,
    },
    NotifyAll {
        except: Option<String>,
        reply: oneshot::Sender<NotifyReport>,
    },
    IsConnected {
        user_name: String,
        reply: oneshot::Sender<bool>,
    },
    ConnectedUserNames {
        reply: oneshot::Sender<Vec<String>>,
    },
    /// Tell every socket except the one whose id is given that the
    /// drift version changed. Each will send its client the new
    /// version, then close. The reply carries how many were told.
    NotifyDrift {
        dont_notify_socket_with_id: Option<u64>,
        reply: oneshot::Sender<usize>,
    },
}