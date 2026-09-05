use my_yrs_lib::yrs_backlinks::YrsBacklinks;
use my_yrs_lib::yrs_wrapper::create_bookmark_of_synced_state;
use my_yrs_lib::{BossOfYrs, YrsActivePages, YrsError};
use protocol::row_col::{Col, Row};
use std::sync::Arc;

#[uniffi::export]
pub fn new_page_row(
    page_id: String,
    is_main_menu_page: bool,
    user_id: String,
) -> Result<Row, YrsError> {
    let page_doc = Arc::new(BossOfYrs::new(user_id));
    let blobbed_page = Arc::clone(&page_doc).snapshot()?;

    let version = create_bookmark_of_synced_state(page_doc)?;

    let active_doc = Arc::new(YrsActivePages::new_empty());
    let page_status = active_doc.snapshot()?;

    Ok(Row {
        cols: vec![
            Col::Text(page_id),
            Col::Blob(blobbed_page),
            Col::Blob(page_status),
            Col::Blob(version),
            Col::Text(if is_main_menu_page {
                "true".to_string()
            } else {
                "false".to_string()
            }),
        ],
    })
}

#[uniffi::export]
pub fn new_every_block_in_existence_row(
    page_that_owns_me: String,
    content: String,
    id_of_block_that_owns: String,
) -> Result<Row, YrsError> {
    Ok(Row {
        cols: vec![
            Col::Text(page_that_owns_me),
            Col::Text(content),
            Col::Text(id_of_block_that_owns),
        ],
    })
}

#[uniffi::export]
pub fn new_uncommitted_diff_row(
    snapshot_of_edit: Vec<u8>,
    edit_enum: Vec<u8>,
    session_id: i64,
    target_id: String,
) -> Result<Row, YrsError> {
    Ok(Row {
        cols: vec![
            Col::Blob(snapshot_of_edit),
            Col::Blob(edit_enum),
            Col::Integer(session_id),
            Col::Text(target_id),
        ],
    })
}

#[uniffi::export]
pub fn new_backlink_row(
    page_that_holds_link_id: String,
    page_being_linked_to_id: String,
) -> Result<Row, YrsError> {
    let backlinks_doc = Arc::new(YrsBacklinks::new_empty());
    let disabled = backlinks_doc.clone().snapshot()?;
    let version = backlinks_doc.create_bookmark_of_synced_state()?;

    Ok(Row {
        cols: vec![
            Col::Text(page_that_holds_link_id),
            Col::Text(page_being_linked_to_id),
            Col::Blob(disabled),
            Col::Blob(version),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::row_col::{Col, Row};

    fn make_page_row(is_main_menu_page: bool) -> Row {
        let user_id = "".to_string();
        new_page_row("test_page".to_string(), is_main_menu_page, user_id).unwrap()
    }

    #[test]
    fn test_new_page_row_both_menu_flags() {
        for menu_flag in [false, true] {
            let row = make_page_row(menu_flag);

            let expected_0 = 5;
            assert_eq!(row.cols.len(), expected_0);

            let expected_1 = Col::Text("test_page".to_string());
            assert_eq!(row.cols[0], expected_1);

            assert!(matches!(row.cols[1], Col::Blob(_)));
            assert!(matches!(row.cols[2], Col::Blob(_)));
            assert!(matches!(row.cols[3], Col::Blob(_)));

            let expected_4 = if menu_flag {
                Col::Text("true".to_string())
            } else {
                Col::Text("false".to_string())
            };
            assert_eq!(row.cols[4], expected_4);
        }
    }

    #[test]
    fn test_new_every_block_in_existence_row() {
        let row = new_every_block_in_existence_row(
            "owner".to_string(),
            "content".to_string(),
            "block_owner".to_string(),
        )
        .unwrap();

        let expected_0 = Col::Text("owner".to_string());
        let expected_1 = Col::Text("content".to_string());
        let expected_2 = Col::Text("block_owner".to_string());

        assert_eq!(row.cols[0], expected_0);
        assert_eq!(row.cols[1], expected_1);
        assert_eq!(row.cols[2], expected_2);
    }

    #[test]
    fn test_new_uncommitted_diff_row() {
        let snapshot = vec![1, 2, 3];
        let edit_enum = vec![4, 5, 6];
        let session_id = 42;
        let target_id = "target".to_string();

        let row = new_uncommitted_diff_row(
            snapshot.clone(),
            edit_enum.clone(),
            session_id,
            target_id.clone(),
        )
        .unwrap();

        let expected_0 = Col::Blob(snapshot);
        let expected_1 = Col::Blob(edit_enum);
        let expected_2 = Col::Integer(session_id);
        let expected_3 = Col::Text(target_id);

        assert_eq!(row.cols[0], expected_0);
        assert_eq!(row.cols[1], expected_1);
        assert_eq!(row.cols[2], expected_2);
        assert_eq!(row.cols[3], expected_3);
    }

    #[test]
    fn test_new_backlink_row() {
        let owner = "page_A".to_string();
        let target = "page_B".to_string();

        let row = new_backlink_row(owner.clone(), target.clone()).unwrap();

        let expected_0 = Col::Text(owner);
        let expected_1 = Col::Text(target);
        assert_eq!(row.cols[0], expected_0);
        assert_eq!(row.cols[1], expected_1);

        assert!(matches!(row.cols[2], Col::Blob(_)));
        assert!(matches!(row.cols[3], Col::Blob(_)));
    }
}
