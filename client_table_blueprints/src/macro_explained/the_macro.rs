#[macro_export]
macro_rules! export_table_names_for_kotlin {
    ($($(#[$m:meta])* pub const $name:ident: &str = $value:expr),* $(,)?) => {
        $(
            $(#[$m])*
            pub const $name: &str = $value;
        )*
        $(
            paste::paste! {
                #[uniffi::export]
                pub fn [<table_name_ $name:lower>]() -> String {
                    $name.to_string()
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! export_column_names_for_kotlin {
    ($table:ident, $($(#[$m:meta])* pub const $name:ident: &str = $value:expr),* $(,)?) => {
        $(
            $(#[$m])*
            pub const $name: &str = $value;
        )*
        $(
            paste::paste! {
                #[uniffi::export]
                pub fn [<$table:lower _ $name:lower>]() -> String {
                    $name.to_string()
                }
            }
        )*
    };
}
