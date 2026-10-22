use crate::handle::NudgeReport;
use error_stuff::ServErr;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

pub enum Command {
    AddClient {
        user_name: String,
        sender: UnboundedSender<()>,
    },
    RemoveClient {
        user_name: String,
    },
    Nudge {
        user_name: String,
        reply: oneshot::Sender<Result<(), ServErr>>,
    },
    IsConnected {
        user_name: String,
        reply: oneshot::Sender<bool>,
    },
    ConnectedUserNames {
        reply: oneshot::Sender<Vec<String>>,
    },
    NudgeAll {
        except: Option<String>,
        reply: oneshot::Sender<NudgeReport>,
    },
}