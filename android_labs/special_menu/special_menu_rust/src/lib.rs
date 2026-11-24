uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn hello() -> String {
    "hello from special_menu_rust".to_string()
}
