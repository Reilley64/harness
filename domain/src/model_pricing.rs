#[derive(Debug, Clone, PartialEq)]
pub struct ModelPricing {
    pub input: Option<f64>,
    pub cached_input: Option<f64>,
    pub output: Option<f64>,
}
