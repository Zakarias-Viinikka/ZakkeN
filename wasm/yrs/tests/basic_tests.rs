use my_yrs_lib::yrs_wrapper::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_block_returns_text_for_existing_block() {
        let boss = Arc::new(BossOfYrs::new());

        let text = "hello read_block".to_string();
        let meta = "some meta".to_string();

        Arc::clone(&boss)
            .insert_new_block(text.clone(), meta)
            .unwrap();

        let block_id = Arc::clone(&boss).get_entire_page().unwrap()[0]
            .id_in_yrs
            .clone();

        let result = Arc::clone(&boss).read_block(block_id).unwrap().unwrap();

        let expected_result = text;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn read_block_returns_none_for_missing_block() {
        let boss = Arc::new(BossOfYrs::new());

        let result = Arc::clone(&boss)
            .read_block("non_existent_id".to_string())
            .unwrap();

        let expected_result = None;
        assert_eq!(result, expected_result);
    }
}
