const KEY: &str = "server_port";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok()?
}

pub fn get_port() -> Option<u16> {
    let s = storage()?;
    let v = s.get_item(KEY).ok()??;
    v.parse::<u16>().ok()
}

pub fn set_port(n: u16) {
    if let Some(s) = storage() {
        let _ = s.set_item(KEY, &n.to_string());
    }
    super::connecting_gate::allow_connecting();
}