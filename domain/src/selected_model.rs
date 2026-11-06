use crate::Model;
use crate::ModelEffort;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedModel {
    pub model: Model,
    pub effort: Option<ModelEffort>,
}

impl SelectedModel {
    pub fn new(model: Model, effort: Option<ModelEffort>) -> Result<Self, &'static str> {
        model.validate_effort(effort)?;
        Ok(Self { model, effort })
    }
}
