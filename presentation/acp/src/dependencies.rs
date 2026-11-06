use application::use_cases::create_conversation;
use application::use_cases::create_message;
use application::use_cases::create_provider_connection;
use application::use_cases::read_conversation;
use application::use_cases::read_model_selection;
use application::use_cases::read_models;
use application::use_cases::read_providers;
use application::use_cases::update_model_selection;

#[fundle::bundle]
pub struct Dependencies {
    pub providers: read_providers::Dependencies,
    pub authentication: create_provider_connection::Dependencies,
    pub models: read_models::Dependencies,
    pub create_conversation: create_conversation::Dependencies,
    pub read_conversation: read_conversation::Dependencies,
    pub create_message: create_message::Dependencies,
    pub read_model_selection: read_model_selection::Dependencies,
    pub update_model_selection: update_model_selection::Dependencies,
}
