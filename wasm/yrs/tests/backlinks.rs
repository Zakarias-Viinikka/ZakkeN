use crdt_test::yrs_backlinks::YrsBacklinks;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_doc_and_add_backlink() {
        let mut backlinks = YrsBacklinks::new_empty();
        backlinks.add_backlink("id_of_the_linker", "id_of_the_linked_page");
        let result = backlinks.get_backlinks_for_page("id_of_the_linked_page");
        assert_eq!(result, vec!["id_of_the_linker"]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn remove_back_link() {
        let mut backlinks = YrsBacklinks::new_empty();
        backlinks.add_backlink("id_of_the_linker", "id_of_the_linked_page");
        let amount_of_links = backlinks
            .get_backlinks_for_page("id_of_the_linked_page")
            .len();
        assert_eq!(amount_of_links, 1);
        backlinks.remove_backlink("id_of_the_linker", "id_of_the_linked_page");
        let amount_of_links = backlinks
            .get_backlinks_for_page("id_of_the_linked_page")
            .len();
        assert_eq!(amount_of_links, 0);
    }

    #[test]
    fn concurrent_adds_to_same_target_page_merge_correctly() {
        // Setup: base state with one backlink A -> B
        let mut base = YrsBacklinks::new_empty();
        base.add_backlink("page_A", "page_B");
        let baseline_snapshot = base.snapshot();

        // Two offline copies from that snapshot
        let mut offline1 = YrsBacklinks::new(baseline_snapshot.clone());
        let mut offline2 = YrsBacklinks::new(baseline_snapshot.clone());

        // Offline1 adds C -> B, offline2 adds D -> B
        offline1.add_backlink("page_C", "page_B");
        offline2.add_backlink("page_D", "page_B");

        // Generate snapshots from each offline doc
        let snapshot1 = offline1.snapshot();
        let snapshot2 = offline2.snapshot();

        // Merge both snapshots back into base
        base.merge_with_snapshot(&snapshot1).unwrap();
        base.merge_with_snapshot(&snapshot2).unwrap();

        // Assert final backlinks for page_B
        let backlinks = base.get_backlinks_for_page("page_B");
        assert!(backlinks.contains(&"page_A".to_string()));
        assert!(backlinks.contains(&"page_C".to_string()));
        assert!(backlinks.contains(&"page_D".to_string()));
        assert_eq!(backlinks.len(), 3);
    }

    #[test]
    fn concurrent_remove_and_add_on_different_keys_merge_correctly() {
        // Setup: base state with backlinks A -> B and C -> B
        let mut base = YrsBacklinks::new_empty();
        base.add_backlink("page_A", "page_B");
        base.add_backlink("page_C", "page_B");
        let baseline_snapshot = base.snapshot();

        // Two offline copies
        let mut offline1 = YrsBacklinks::new(baseline_snapshot.clone());
        let mut offline2 = YrsBacklinks::new(baseline_snapshot.clone());

        // Offline1 removes A -> B, offline2 adds D -> B
        offline1.remove_backlink("page_A", "page_B");
        offline2.add_backlink("page_D", "page_B");

        // Snapshots
        let snapshot1 = offline1.snapshot();
        let snapshot2 = offline2.snapshot();

        // Merge into base
        base.merge_with_snapshot(&snapshot1).unwrap();
        base.merge_with_snapshot(&snapshot2).unwrap();

        // Expected: A is gone, C remains, D is added
        let backlinks = base.get_backlinks_for_page("page_B");
        assert!(!backlinks.contains(&"page_A".to_string()));
        assert!(backlinks.contains(&"page_C".to_string()));
        assert!(backlinks.contains(&"page_D".to_string()));
        assert_eq!(backlinks.len(), 2);
    }
}
