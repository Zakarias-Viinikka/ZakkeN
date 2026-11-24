/// Error returned by a link command. Same three fields everywhere: what went wrong,
/// which method, which file.
#[derive(Clone, Debug)]
pub struct OnLinkClickError {
    pub err_msg: String,
    pub method: String,
    pub filepath: String,
}
