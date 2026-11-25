#[derive(Clone)]
pub struct TitleMatch {
    pub title: String,
    pub page_id: String,
}

#[derive(Clone)]
pub enum MenuOption {
    MakeNewPageAndLinkToIt,
    LinkToExistingPage,
}
