#[derive(Debug)]
pub struct Output {
    pub entries: Vec<domain::UsageEntry>,
    pub unassigned_native_runs: usize,
    pub unknown_history_sources: Vec<String>,
}
