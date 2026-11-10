use protocol::serialization::Convert;
use server_protocol::{Request, WipeDatasetPayload};

pub enum ClientCommand {
    WipeDataset,
}

pub fn build_request(command: ClientCommand) -> (Request, Vec<u8>) {
    match command {
        ClientCommand::WipeDataset => build_wipe_dataset_request(),
    }
}

fn build_wipe_dataset_request() -> (Request, Vec<u8>) {
    let payload = WipeDatasetPayload {
        drift_solution: None,
    };
    (Request::WipeDataset, payload.to_payload())
}
