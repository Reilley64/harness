pub struct Output {
    pub revision: Option<u64>,
    pub selected: Option<domain::InstructionSnapshot>,
    pub current: Option<domain::InstructionSnapshot>,
    pub current_run: Option<String>,
    pub historical_unknown: Vec<String>,
    pub resolution: domain::InstructionResolution,
}
