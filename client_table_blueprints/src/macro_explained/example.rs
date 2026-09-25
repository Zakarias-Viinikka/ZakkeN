use crate::{export_column_names_for_kotlin, export_table_names_for_kotlin};

export_table_names_for_kotlin! {
    pub const BABLE_LADE: &str = "namle_tabe",
}

export_column_names_for_kotlin! {bable_lade,
    pub const COL1: &str = "name",
    pub const COL2: &str = "age",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consts_exist() {
        assert_eq!(BABLE_LADE, "namle_tabe");
        assert_eq!(COL1, "name");
        assert_eq!(COL2, "age");
    }

    #[test]
    fn getters_return_const_values() {
        assert_eq!(table_name_bable_lade(), BABLE_LADE);
        assert_eq!(bable_lade_col1(), COL1);
        assert_eq!(bable_lade_col2(), COL2);
    }
}
