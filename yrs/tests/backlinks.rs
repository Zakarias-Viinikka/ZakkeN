use my_yrs_lib::yrs_backlinks::YrsBacklinks;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_doc_and_add_backlink() {
        let backlinks = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&backlinks)
            .add_backlink(
                "id_of_the_linker".to_string(),
                "id_of_the_linked_page".to_string(),
            )
            .unwrap();
        let result = Arc::clone(&backlinks)
            .get_backlinks_for_page("id_of_the_linked_page".to_string())
            .unwrap();
        assert_eq!(result, vec!["id_of_the_linker"]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn remove_back_link() {
        let backlinks = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&backlinks)
            .add_backlink(
                "id_of_the_linker".to_string(),
                "id_of_the_linked_page".to_string(),
            )
            .unwrap();
        let amount = Arc::clone(&backlinks)
            .get_backlinks_for_page("id_of_the_linked_page".to_string())
            .unwrap()
            .len();
        assert_eq!(amount, 1);

        Arc::clone(&backlinks)
            .remove_backlink(
                "id_of_the_linker".to_string(),
                "id_of_the_linked_page".to_string(),
            )
            .unwrap();
        let amount = Arc::clone(&backlinks)
            .get_backlinks_for_page("id_of_the_linked_page".to_string())
            .unwrap()
            .len();
        assert_eq!(amount, 0);
    }

    #[test]
    fn concurrent_adds_to_same_target_page_merge_correctly() {
        let base = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&base)
            .add_backlink("page_A".to_string(), "page_B".to_string())
            .unwrap();
        let baseline_snapshot = Arc::clone(&base).snapshot().unwrap();

        let offline1 = Arc::new(YrsBacklinks::new(baseline_snapshot.clone()).unwrap());
        let offline2 = Arc::new(YrsBacklinks::new(baseline_snapshot.clone()).unwrap());

        Arc::clone(&offline1)
            .add_backlink("page_C".to_string(), "page_B".to_string())
            .unwrap();
        Arc::clone(&offline2)
            .add_backlink("page_D".to_string(), "page_B".to_string())
            .unwrap();

        let snapshot1 = Arc::clone(&offline1).snapshot().unwrap();
        let snapshot2 = Arc::clone(&offline2).snapshot().unwrap();

        Arc::clone(&base).merge_with_snapshot(snapshot1).unwrap();
        Arc::clone(&base).merge_with_snapshot(snapshot2).unwrap();

        let backlinks = Arc::clone(&base)
            .get_backlinks_for_page("page_B".to_string())
            .unwrap();
        assert!(backlinks.contains(&"page_A".to_string()));
        assert!(backlinks.contains(&"page_C".to_string()));
        assert!(backlinks.contains(&"page_D".to_string()));
        assert_eq!(backlinks.len(), 3);
    }

    #[test]
    fn concurrent_remove_and_add_on_different_keys_merge_correctly() {
        let base = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&base)
            .add_backlink("page_A".to_string(), "page_B".to_string())
            .unwrap();
        Arc::clone(&base)
            .add_backlink("page_C".to_string(), "page_B".to_string())
            .unwrap();
        let baseline_snapshot = Arc::clone(&base).snapshot().unwrap();

        let offline1 = Arc::new(YrsBacklinks::new(baseline_snapshot.clone()).unwrap());
        let offline2 = Arc::new(YrsBacklinks::new(baseline_snapshot.clone()).unwrap());

        Arc::clone(&offline1)
            .remove_backlink("page_A".to_string(), "page_B".to_string())
            .unwrap();
        Arc::clone(&offline2)
            .add_backlink("page_D".to_string(), "page_B".to_string())
            .unwrap();

        let snapshot1 = Arc::clone(&offline1).snapshot().unwrap();
        let snapshot2 = Arc::clone(&offline2).snapshot().unwrap();

        Arc::clone(&base).merge_with_snapshot(snapshot1).unwrap();
        Arc::clone(&base).merge_with_snapshot(snapshot2).unwrap();

        let backlinks = Arc::clone(&base)
            .get_backlinks_for_page("page_B".to_string())
            .unwrap();
        assert!(!backlinks.contains(&"page_A".to_string()));
        assert!(backlinks.contains(&"page_C".to_string()));
        assert!(backlinks.contains(&"page_D".to_string()));
        assert_eq!(backlinks.len(), 2);
    }
}
