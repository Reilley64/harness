#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpTransport {
    Disabled,
    Stdio {
        command: String,
        args: Vec<String>,
    },
    Http {
        url: String,
        headers: std::collections::BTreeMap<String, String>,
    },
}
