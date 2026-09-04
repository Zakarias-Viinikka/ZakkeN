use my_yrs_lib::yrs_active_pages::YrsActivePages;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_active_for_unknown_page() {
        let active_pages = Arc::new(YrsActivePages::new_empty());
        let result = Arc::clone(&active_pages)
            .is_page_active("unknown_page".to_string())
            .unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn mark_page_active_sets_true() {
        let active_pages = Arc::new(YrsActivePages::new_empty());
        Arc::clone(&active_pages)
            .mark_page_active("page_A".to_string())
            .unwrap();
        let is_active = Arc::clone(&active_pages)
            .is_page_active("page_A".to_string())
            .unwrap();
        assert_eq!(is_active, true);
    }

    #[test]
    fn mark_page_deleted_sets_false() {
        let active_pages = Arc::new(YrsActivePages::new_empty());
        Arc::clone(&active_pages)
            .mark_page_deleted("page_A".to_string())
            .unwrap();
        let is_active = Arc::clone(&active_pages)
            .is_page_active("page_A".to_string())
            .unwrap();
        assert_eq!(is_active, false);
    }

    #[test]
    fn overwrite_last_write_wins() {
        let active_pages = Arc::new(YrsActivePages::new_empty());
        Arc::clone(&active_pages)
            .mark_page_active("page_A".to_string())
            .unwrap();
        Arc::clone(&active_pages)
            .mark_page_deleted("page_A".to_string())
            .unwrap();
        let is_active = Arc::clone(&active_pages)
            .is_page_active("page_A".to_string())
            .unwrap();
        assert_eq!(is_active, false);

        Arc::clone(&active_pages)
            .mark_page_active("page_A".to_string())
            .unwrap();
        let is_active = Arc::clone(&active_pages)
            .is_page_active("page_A".to_string())
            .unwrap();
        assert_eq!(is_active, true);
    }

    #[test]
    fn snapshot_and_merge_basic() {
        let base = Arc::new(YrsActivePages::new_empty());
        let baseline_snapshot = Arc::clone(&base).snapshot().unwrap();

        let offline1 = Arc::new(YrsActivePages::new(baseline_snapshot.clone()).unwrap());
        Arc::clone(&offline1)
            .mark_page_deleted("page_A".to_string())
            .unwrap();

        let offline2 = Arc::new(YrsActivePages::new(baseline_snapshot.clone()).unwrap());
        Arc::clone(&offline2)
            .mark_page_deleted("page_B".to_string())
            .unwrap();

        let snapshot1 = Arc::clone(&offline1).snapshot().unwrap();
        let snapshot2 = Arc::clone(&offline2).snapshot().unwrap();

        Arc::clone(&base).merge_with_snapshot(snapshot1).unwrap();
        Arc::clone(&base).merge_with_snapshot(snapshot2).unwrap();

        let is_a_active = Arc::clone(&base)
            .is_page_active("page_A".to_string())
            .unwrap();
        let is_b_active = Arc::clone(&base)
            .is_page_active("page_B".to_string())
            .unwrap();
        assert_eq!(is_a_active, false);
        assert_eq!(is_b_active, false);
    }
}
