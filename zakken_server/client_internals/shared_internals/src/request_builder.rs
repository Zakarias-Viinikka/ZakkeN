use server_protocol::Request;

pub enum ClientCommand {
    WipeDataset,
}

pub fn build_request(command: ClientCommand) -> (Request, Vec<u8>) {
    match command {
        ClientCommand::WipeDataset => build_wipe_dataset_request(),
    }
}

fn build_wipe_dataset_request() -> (Request, Vec<u8>) {
    (Request::WipeDataset, Vec::new())
}
