//! Command `[[PageNav|page id]]`: open that page. `rest` is the page id.
use crate::complex_state::navigation_request::plexset_navigation_request;

use super::on_link_click_error::OnLinkClickError;

pub fn run_page_nav(rest: &str) -> Result<(), OnLinkClickError> {
    let page_id = rest.trim();
    if page_id.is_empty() {
        return Err(OnLinkClickError {
            err_msg: "PageNav needs a page id after the |, got nothing".to_string(),
            method: "run_page_nav".to_string(),
            filepath: file!().to_string(),
        });
    }
    plexset_navigation_request(format!("/MainContent/{page_id}"));
    Ok(())
}
