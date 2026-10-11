# ui_actions_helper

## get_page_is_disabled

Lets the caller know if a page is disabled, so navigation knows whether a
link should work or be visible.

## get_active_pages_blob

Gets the blob from the pages table so it can get the yrs doc that
represents whether a page is disabled or not. Disable page retrieves it
so it can change the page status to disabled and then insert the new
blob.
