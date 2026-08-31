use crdt_test::yrs_active_pages::YrsActivePages;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_active_for_unknown_page() {
        let active_pages = YrsActivePages::new_empty();
        assert_eq!(active_pages.is_page_active("unknown_page"), true);
    }

    #[test]
    fn mark_page_active_sets_true() {
        let mut active_pages = YrsActivePages::new_empty();
        active_pages.mark_page_active("page_A");
        assert_eq!(active_pages.is_page_active("page_A"), true);
    }

    #[test]
    fn mark_page_deleted_sets_false() {
        let mut active_pages = YrsActivePages::new_empty();
        active_pages.mark_page_deleted("page_A");
        assert_eq!(active_pages.is_page_active("page_A"), false);
    }

    #[test]
    fn overwrite_last_write_wins() {
        let mut active_pages = YrsActivePages::new_empty();
        active_pages.mark_page_active("page_A");
        active_pages.mark_page_deleted("page_A");
        // Deleted should win because it's last
        assert_eq!(active_pages.is_page_active("page_A"), false);

        // Now mark active again
        active_pages.mark_page_active("page_A");
        assert_eq!(active_pages.is_page_active("page_A"), true);
    }

    #[test]
    fn snapshot_and_merge_basic() {
        // Base state: both pages active (no entries)
        let mut base = YrsActivePages::new_empty();
        let baseline_snapshot = base.snapshot();

        // Offline copy 1 marks page_A deleted
        let mut offline1 = YrsActivePages::new(baseline_snapshot.clone());
        offline1.mark_page_deleted("page_A");

        // Offline copy 2 marks page_B deleted
        let mut offline2 = YrsActivePages::new(baseline_snapshot.clone());
        offline2.mark_page_deleted("page_B");

        // Get snapshots
        let snapshot1 = offline1.snapshot();
        let snapshot2 = offline2.snapshot();

        // Merge both into base
        base.merge_with_snapshot(&snapshot1).unwrap();
        base.merge_with_snapshot(&snapshot2).unwrap();

        // Now both pages should be inactive
        assert_eq!(base.is_page_active("page_A"), false);
        assert_eq!(base.is_page_active("page_B"), false);
    }
}
