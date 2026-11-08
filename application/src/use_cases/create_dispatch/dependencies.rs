use crate::use_cases::create_message;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) worker: create_message::Dependencies,
}
