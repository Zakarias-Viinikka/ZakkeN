use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;

const FILEPATH: &str = "zakken.sqlite";

pub fn new_liver() -> Result<LiveForever, DbError> {
    let path = read_path_from_args().unwrap_or_else(|| FILEPATH.to_string());
    LiveForever::new(&path)
}

fn read_path_from_args() -> Option<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(position) = args.iter().position(|a| a == "zzpath") else {
        return None;
    };
    let Some(value) = args.get(position + 1) else {
        panic!("zzpath needs a value");
    };
    Some(value.clone())
}
