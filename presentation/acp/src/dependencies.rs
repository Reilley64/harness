use application::use_cases::create_agent;
use application::use_cases::create_conversation;
use application::use_cases::create_message;
use application::use_cases::create_provider_connection;
use application::use_cases::read_agent;
use application::use_cases::read_conversation;
use application::use_cases::read_deciders;
use application::use_cases::read_decision;
use application::use_cases::read_guidance;
use application::use_cases::read_model_allowlist;
use application::use_cases::read_model_selection;
use application::use_cases::read_models;
use application::use_cases::read_providers;
use application::use_cases::read_work;
use application::use_cases::update_agent;
use application::use_cases::update_decider;
use application::use_cases::update_decision;
use application::use_cases::update_guidance;
use application::use_cases::update_model_allowlist;
use application::use_cases::update_model_selection;
use application::use_cases::update_work;

#[fundle::bundle]
pub struct Dependencies {
    pub read_decision: read_decision::Dependencies,
    pub update_decision: update_decision::Dependencies,
    pub read_deciders: read_deciders::Dependencies,
    pub update_decider: update_decider::Dependencies,
    pub read_work: read_work::Dependencies,
    pub update_work: update_work::Dependencies,
    pub read_model_allowlist: read_model_allowlist::Dependencies,
    pub update_model_allowlist: update_model_allowlist::Dependencies,
    pub providers: read_providers::Dependencies,
    pub authentication: create_provider_connection::Dependencies,
    pub models: read_models::Dependencies,
    pub create_agent: create_agent::Dependencies,
    pub read_agent: read_agent::Dependencies,
    pub read_guidance: read_guidance::Dependencies,
    pub update_guidance: update_guidance::Dependencies,
    pub update_agent: update_agent::Dependencies,
    pub create_conversation: create_conversation::Dependencies,
    pub read_conversation: read_conversation::Dependencies,
    pub create_message: create_message::Dependencies,
    pub read_model_selection: read_model_selection::Dependencies,
    pub update_model_selection: update_model_selection::Dependencies,
}
