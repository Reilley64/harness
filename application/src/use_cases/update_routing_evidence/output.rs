pub struct Output {
    pub evidence: Vec<(String, Vec<domain::RoutingEvidence>)>,
    pub diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
    pub checked_at: u64,
}
