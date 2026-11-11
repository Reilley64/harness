#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishWorkArguments {
    pub work_id: String,
    pub intent: String,
    pub path: String,
    pub revision: Option<u64>,
}
