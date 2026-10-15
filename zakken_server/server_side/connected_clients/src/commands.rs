use crate::handle::FanOutReport;
use error_stuff::ServErr;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

pub enum Command {
    AddClient {
        user_name: String,
        sender: UnboundedSender<i64>,
    },
    RemoveClient {
        user_name: String,
    },
    SendTo {
        user_name: String,
        letter_id: i64,
        reply: oneshot::Sender<Result<(), ServErr>>,
    },
    IsConnected {
        user_name: String,
        reply: oneshot::Sender<bool>,
    },
    ConnectedUserNames {
        reply: oneshot::Sender<Vec<String>>,
    },
    FanOut {
        letter_id: i64,
        except: Option<String>,
        reply: oneshot::Sender<FanOutReport>,
    },
}
