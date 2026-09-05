use my_yrs_lib::yrs_backlinks::YrsBacklinks;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_disabled() {
        let backlinks = Arc::new(YrsBacklinks::new_empty());
        let result = Arc::clone(&backlinks).is_disabled().unwrap();
        let expected = false;
        assert_eq!(result, expected);
    }

    #[test]
    fn set_disabled_true() {
        let backlinks = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&backlinks).set_disabled(true).unwrap();
        let result = Arc::clone(&backlinks).is_disabled().unwrap();
        let expected = true;
        assert_eq!(result, expected);
    }

    #[test]
    fn set_disabled_false_after_true() {
        let backlinks = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&backlinks).set_disabled(true).unwrap();
        Arc::clone(&backlinks).set_disabled(false).unwrap();
        let result = Arc::clone(&backlinks).is_disabled().unwrap();
        let expected = false;
        assert_eq!(result, expected);
    }

    #[test]
    fn merge_snapshot_updates_state() {
        let base = Arc::new(YrsBacklinks::new_empty());
        Arc::clone(&base).set_disabled(false).unwrap();
        let baseline_snapshot = Arc::clone(&base).snapshot().unwrap();

        let offline = Arc::new(YrsBacklinks::new(baseline_snapshot).unwrap());
        Arc::clone(&offline).set_disabled(true).unwrap();
        let offline_snapshot = Arc::clone(&offline).snapshot().unwrap();

        Arc::clone(&base)
            .merge_with_snapshot(offline_snapshot)
            .unwrap();

        let result = Arc::clone(&base).is_disabled().unwrap();
        let expected = true;
        assert_eq!(result, expected);
    }
}
