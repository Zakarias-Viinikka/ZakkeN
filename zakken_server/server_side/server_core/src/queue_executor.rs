use crate::allowed_requests::do_request;
use crate::server_state::ServerState;
use connected_clients::ConnectedClients;
use error_stuff::ServerLogEntry;
use queue::{Job, JobReply};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

/// The single task that owns every request. It takes jobs (important
/// channel first, then normal), runs the handler, and sends the result
/// back through the job's oneshot. One at a time, so no two requests
/// ever interleave.
///
/// It also drains `log_rx`: anything that wants to record an entry sends
/// a `ServerLogEntry` down that channel, and this loop writes the row.
/// That's how code that doesn't own the db can still log — the executor
/// is the only thing that talks to sqlite.
pub fn start(
    mut important_rx: UnboundedReceiver<Job>,
    mut normal_rx: UnboundedReceiver<Job>,
    mut log_rx: UnboundedReceiver<ServerLogEntry>,
    state: Arc<Mutex<ServerState>>,
    clients: ConnectedClients,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                Some(job) = important_rx.recv() => {
                    run_job(job, &state, &clients).await;
                }
                Some(job) = normal_rx.recv() => {
                    run_job(job, &state, &clients).await;
                }
                Some(entry) = log_rx.recv() => {
                    write_log(entry, &state);
                }
                else => break,
            }
        }
    });
}

async fn run_job(job: Job, state: &Arc<Mutex<ServerState>>, clients: &ConnectedClients) {
    let outcome = do_request(
        &job.request,
        job.payload,
        state,
        clients,
        job.sender.as_deref(),
    )
    .await;

    let reply = JobReply {
        bytes: outcome.bytes,
        just_identified: outcome.just_identified,
        incoming_letters: outcome.incoming_letters,
    };
    let _ = job.reply.send(reply);
}

fn write_log(entry: ServerLogEntry, state: &Arc<Mutex<ServerState>>) {
    let locked = state.lock().unwrap();
    if let Err(e) = db_core::db::operations_logs::insert_log(
        &locked.liver,
        entry.timestamp,
        entry.level.as_str(),
        entry.category.as_str(),
        &entry.source,
        &entry.session_id,
        &entry.message,
    ) {
        // Printed directly, not through log_error: sending another entry
        // down the same channel would loop forever if writes keep failing.
        println!(
            "[{}] server: ERROR in queue_executor::write_log: couldn't write a log row: {e:?}",
            crate::logging::timestamp()
        );
    }
}