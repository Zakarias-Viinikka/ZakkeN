use protocol::serialization::Convert;
use server_protocol::{
    GetPagesByIdsPayload, ReconcileLettersPayload, Request, WipeDatasetPayload,
};

pub enum ClientCommand {
    WipeDataset,
    GetPageFingerprints,
    GetPagesByIds(Vec<String>),
    GetReconcileVersion,
    ReconcileLetters {
        target_drift_version: String,
        target_reconcile_version: String,
        letters: Vec<Vec<u8>>,
    },
}

pub fn build_request(command: ClientCommand) -> (Request, Vec<u8>) {
    match command {
        ClientCommand::WipeDataset => build_wipe_dataset_request(),
        ClientCommand::GetPageFingerprints => (Request::GetPageFingerprints, Vec::new()),
        ClientCommand::GetPagesByIds(page_ids) => {
            let payload = GetPagesByIdsPayload { page_ids };
            (Request::GetPagesByIds, payload.to_payload())
        }
        ClientCommand::GetReconcileVersion => (Request::GetReconcileVersion, Vec::new()),
        ClientCommand::ReconcileLetters {
            target_drift_version,
            target_reconcile_version,
            letters,
        } => {
            let payload = ReconcileLettersPayload {
                target_drift_version,
                target_reconcile_version,
                letters,
            };
            (Request::ReconcileLetters, payload.to_payload())
        }
    }
}

fn build_wipe_dataset_request() -> (Request, Vec<u8>) {
    let payload = WipeDatasetPayload {
        drift_solution: None,
    };
    (Request::WipeDataset, payload.to_payload())
}
