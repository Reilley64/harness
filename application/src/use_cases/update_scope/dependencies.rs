use crate::use_cases::create_message;

#[derive(Clone)]
pub struct Dependencies {
    pub(crate) worker: create_message::Dependencies,
}

impl Dependencies {
    pub fn new(worker: create_message::Dependencies) -> Self {
        Self { worker }
    }
}
