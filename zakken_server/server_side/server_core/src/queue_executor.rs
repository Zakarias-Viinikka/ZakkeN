use crate::allowed_requests::do_request;
use db_wrapper::android_mascot::LiveForever;
use crate::telephone::Telephone;
use error_stuff::ServerLogEntry;
use queue::Job;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

pub fn start(
    mut important_rx: UnboundedReceiver<Job>,
    mut normal_rx: UnboundedReceiver<Job>,
    mut log_rx: UnboundedReceiver<ServerLogEntry>,
    state: Arc<Mutex<LiveForever>>,
    telephone: Telephone,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                Some(job) = important_rx.recv() => {
                    run_job(job, &state, &telephone).await;
                }
                Some(job) = normal_rx.recv() => {
                    run_job(job, &state, &telephone).await;
                }
                Some(entry) = log_rx.recv() => {
                    write_log(entry, &state);
                }
                else => break,
            }
        }
    });
}

async fn run_job(job: Job, state: &Arc<Mutex<LiveForever>>, telephone: &Telephone) {
    let bytes = do_request(
        &job.request,
        job.payload,
        state,
        telephone,
        job.socket_id,
    )
    .await;

    let _ = job.reply.send(bytes);
}

fn write_log(entry: ServerLogEntry, state: &Arc<Mutex<LiveForever>>) {
    let locked = state.lock().unwrap();
    if let Err(e) = db_core::db::operations_logs::insert_log(
        &locked,
        entry.timestamp,
        entry.level.as_str(),
        entry.category.as_str(),
        &entry.source,
        &entry.session_id,
        &entry.message,
    ) {
        println!(
            "[{}] server: ERROR in queue_executor::write_log: couldn't write a log row: {e:?}",
            crate::logging::timestamp()
        );
    }
}