use super::Error;

pub struct Output {
    pub diagnostics: Vec<Error>,
    pub models: Vec<(domain::Model, Option<String>)>,
    pub selected: Option<String>,
}
