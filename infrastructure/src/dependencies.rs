use crate::create_agent;
use crate::create_conversation;
use crate::create_coordinator;
use crate::create_file;
use crate::create_mcp_call;
use crate::create_model_step_factory;
use crate::create_native_decision;
use crate::create_process;
use crate::create_tool_output;
use crate::lock_conversation;
use crate::mcp::Connections;
use crate::read_agent;
use crate::read_code_matches;
use crate::read_conversation;
use crate::read_decider;
use crate::read_decision;
use crate::read_directory;
use crate::read_file;
use crate::read_instructions;
use crate::read_model_allowlist;
use crate::read_model_selection;
use crate::read_paths;
use crate::read_provider_connection;
use crate::read_setup;
use crate::read_setup_draft;
use crate::read_text_matches;
use crate::read_tool_invocation;
use crate::read_tools;
use crate::read_web_search;
use crate::read_work;
use crate::read_work_proposal;
use crate::update_conversation;
use crate::update_decider;
use crate::update_decision;
use crate::update_file;
use crate::update_mcp_configuration;
use crate::update_model_allowlist;
use crate::update_model_selection;
use crate::update_setup_draft;
use crate::update_work;

use std::time::Duration;

use application::ports;

use crate::create_provider_connection_factory::create_provider_connection_factory;
use crate::read_models::read_models;
use crate::read_providers::read_providers;
use crate::settings::Settings;

#[fundle::bundle]
pub struct Dependencies {
    settings: Settings,
    create_assignment: application::use_cases::create_assignment::Dependencies,
    read_decision: ports::ReadDecision,
    update_decision: ports::UpdateDecision,
    create_native_decision: ports::CreateNativeDecision,
    read_decider: ports::ReadDecider,
    update_decider: ports::UpdateDecider,
    create_work: application::use_cases::create_work::Dependencies,
    read_work: ports::ReadWork,
    update_work: ports::UpdateWork,
    read_work_proposal: ports::ReadWorkProposal,
    create_coordinator: ports::CreateCoordinator,
    read_file: ports::ReadFile,
    create_process: ports::CreateProcess,
    update_file: ports::UpdateFile,
    create_file: ports::CreateFile,
    read_directory: ports::ReadDirectory,
    read_paths: ports::ReadPaths,
    read_text_matches: ports::ReadTextMatches,
    read_code_matches: ports::ReadCodeMatches,
    read_web_search: ports::ReadWebSearch,
    mcp_connections: Connections,
    update_mcp_configuration: ports::UpdateMcpConfiguration,
    create_mcp_call: ports::CreateMcpCall,
    read_tools: ports::ReadTools,
    read_tool_invocation: ports::ReadToolInvocation,
    create_tool_output: ports::CreateToolOutput,
    read_model_allowlist: ports::ReadModelAllowlist,
    update_model_allowlist: ports::UpdateModelAllowlist,
    read_model_selection: ports::ReadModelSelection,
    update_model_selection: ports::UpdateModelSelection,
    create_agent: ports::CreateAgent,
    read_setup_draft: ports::ReadSetupDraft,
    update_setup_draft: ports::UpdateSetupDraft,
    read_setup: ports::ReadSetup,
    read_agent: ports::ReadAgent,
    read_instructions: ports::ReadInstructions,
    create_conversation: ports::CreateConversation,
    read_conversation: ports::ReadConversation,
    update_conversation: ports::UpdateConversation,
    lock_conversation: ports::LockConversation,
    read_provider_connection: ports::ReadProviderConnection,
    create_model_step_factory: ports::CreateModelStepFactory,
    create_provider_connection_factory: ports::CreateProviderConnectionFactory,
    read_models: ports::ReadModels,
    read_providers: ports::ReadProviders,
}

impl Dependencies {
    pub fn build(settings: Settings) -> Self {
        Self::builder()
            .settings(|_| settings.clone())
            .mcp_connections(|_| Connections::new())
            .update_mcp_configuration(|deps| {
                let settings: &Settings = deps.as_ref();
                let connections: &Connections = deps.as_ref();
                let update = update_mcp_configuration::update_mcp_configuration(
                    &settings.app_data_path,
                    &settings.mcp.registry_url,
                    connections,
                );
                ports::UpdateMcpConfiguration::new(move |workspace, action, cancellation| {
                    let future = update(workspace, action, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .create_mcp_call(|deps| {
                let connections: &Connections = deps.as_ref();
                let configuration: &ports::UpdateMcpConfiguration = deps.as_ref();
                let create = create_mcp_call::create_mcp_call(connections, configuration.clone());
                ports::CreateMcpCall::new(move |workspace, arguments, cancellation| {
                    let future = create(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .create_provider_connection_factory(|deps| {
                let settings: &Settings = deps.as_ref();
                let credentials_path = settings.app_data_path.join("auth");
                let settings = settings.authentication.clone();
                ports::CreateProviderConnectionFactory::new(move |provider_id| {
                    let create = create_provider_connection_factory(
                        &settings,
                        &credentials_path,
                        &provider_id,
                    )?;
                    Ok(ports::CreateProviderConnection::new(
                        move |present_authorization| {
                            let result = create(present_authorization);
                            Box::pin(async move {
                                let connection = result.await?;
                                Ok(connection)
                            })
                        },
                    ))
                })
            })
            .read_decision(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation =
                    read_decision::read_decision(&settings.app_data_path.join("decisions"));
                ports::ReadDecision::new(move |id| {
                    let result = implementation(id);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .update_decision(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation =
                    update_decision::update_decision(&settings.app_data_path.join("decisions"));
                ports::UpdateDecision::new(move |decision, revision| {
                    let result = implementation(decision, revision);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_decider(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_decider::read_decider(
                    &settings.app_data_path.join("preferences/decider.json"),
                );
                ports::ReadDecider::new(move || {
                    let result = implementation();
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .update_decider(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = update_decider::update_decider(
                    &settings.app_data_path.join("preferences/decider.json"),
                );
                ports::UpdateDecider::new(move |model_id| {
                    let result = implementation(model_id);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_work(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_work::read_work(&settings.app_data_path.join("work"));
                ports::ReadWork::new(move |id| {
                    let result = implementation(id);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .update_work(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = update_work::update_work(&settings.app_data_path.join("work"));
                ports::UpdateWork::new(move |work, revision| {
                    let result = implementation(work, revision);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_work_proposal(|_| {
                let implementation = read_work_proposal::read_work_proposal();
                ports::ReadWorkProposal::new(move |workspace, path, cancellation| {
                    let result = implementation(workspace, path, cancellation);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .create_coordinator(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = create_coordinator::create_coordinator(
                    &settings.app_data_path.join("conversations"),
                );
                ports::CreateCoordinator::new(
                    move |work, approval, model, connection, instructions, adopt_default| {
                        let result = implementation(
                            work,
                            approval,
                            model,
                            connection,
                            instructions,
                            adopt_default,
                        );
                        Box::pin(async move { Ok(result.await?) })
                    },
                )
            })
            .create_agent(|deps| {
                let settings: &Settings = deps.as_ref();
                let create =
                    create_agent::create_agent(&settings.app_data_path.join("conversations"));
                ports::CreateAgent::new(move |model, effort, connection, workspace, role| {
                    let result = create(model, effort, connection, workspace, role);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_setup_draft(|_| {
                let read = read_setup_draft::read_setup_draft();
                ports::ReadSetupDraft::new(
                    move |workspace, arguments, proposed_wiki, cancellation| {
                        let result = read(workspace, arguments, proposed_wiki, cancellation);
                        Box::pin(async move { Ok(result.await?) })
                    },
                )
            })
            .update_setup_draft(|deps| {
                let settings: &Settings = deps.as_ref();
                let update = update_setup_draft::update_setup_draft(&settings.app_data_path);
                ports::UpdateSetupDraft::new(move |draft, cancellation| {
                    let result = update(draft, cancellation);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_setup(|_| {
                let read = read_setup::read_setup();
                ports::ReadSetup::new(move |workspace, cancellation| {
                    let result = read(workspace, cancellation);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_agent(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_agent::read_agent(&settings.app_data_path.join("conversations"));
                ports::ReadAgent::new(move |workspace, role| {
                    let result = read(workspace, role);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_instructions(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_instructions::read_instructions(&settings.app_data_path);
                ports::ReadInstructions::new(move |workspace, phase, scopes, references| {
                    let result = read(workspace, phase, scopes, references);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .create_conversation(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = create_conversation::create_conversation(
                    &settings.app_data_path.join("conversations"),
                );
                ports::CreateConversation::new(move |model, effort, connection, workspace| {
                    let result = implementation(model, effort, connection, workspace);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_conversation(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_conversation::read_conversation(
                    &settings.app_data_path.join("conversations"),
                );
                ports::ReadConversation::new(move |id| {
                    let result = implementation(id);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .update_conversation(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = update_conversation::update_conversation(
                    &settings.app_data_path.join("conversations"),
                );
                ports::UpdateConversation::new(move |conversation| {
                    let result = implementation(conversation);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .lock_conversation(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = lock_conversation::lock_conversation(
                    &settings.app_data_path.join("conversations"),
                );
                ports::LockConversation::new(move |id| {
                    let result = implementation(id);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_provider_connection(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_provider_connection::read_provider_connection(
                    &settings.app_data_path.join("auth"),
                );
                ports::ReadProviderConnection::new(move |provider_id| {
                    let result = implementation(provider_id);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_model_allowlist(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_model_allowlist::read_model_allowlist(
                    &settings
                        .app_data_path
                        .join("preferences/model-allowlist.json"),
                );
                ports::ReadModelAllowlist::new(move || {
                    let future = read();
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .update_model_allowlist(|deps| {
                let settings: &Settings = deps.as_ref();
                let update = update_model_allowlist::update_model_allowlist(
                    &settings
                        .app_data_path
                        .join("preferences/model-allowlist.json"),
                );
                ports::UpdateModelAllowlist::new(move |operation| {
                    let future = update(operation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_models(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_models(
                    &settings.models.url,
                    &settings.app_data_path.join("models.json"),
                    Duration::from_secs(settings.models.memory_ttl_seconds),
                );
                ports::ReadModels::new(move || {
                    let future = read();
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .create_model_step_factory(|deps| {
                let settings: &Settings = deps.as_ref();
                let read_model_allowlist: &ports::ReadModelAllowlist = deps.as_ref();
                let read_model_allowlist = read_model_allowlist.clone();
                let reply_settings = settings.replies.clone();
                let authentication = settings.authentication.clone();
                let read_models: &ports::ReadModels = deps.as_ref();
                let read_models = read_models.clone();
                let read_work: &ports::ReadWork = deps.as_ref();
                let read_work = read_work.clone();
                let credentials_path = settings.app_data_path.join("auth");
                ports::CreateModelStepFactory::new(move |provider_id| {
                    let create = create_model_step_factory::create_model_step_factory(
                        &reply_settings,
                        &authentication,
                        &credentials_path,
                        &provider_id,
                        read_model_allowlist.clone(),
                        read_models.clone(),
                        read_work.clone(),
                    )?;
                    Ok(create)
                })
            })
            .read_model_selection(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_model_selection::read_model_selection(
                    &settings
                        .app_data_path
                        .join("preferences/model-selection.json"),
                );
                ports::ReadModelSelection::new(move || {
                    let result = read();
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .update_model_selection(|deps| {
                let settings: &Settings = deps.as_ref();
                let update = update_model_selection::update_model_selection(
                    &settings
                        .app_data_path
                        .join("preferences/model-selection.json"),
                );
                ports::UpdateModelSelection::new(move |selection| {
                    let result = update(selection);
                    Box::pin(async move { Ok(result.await?) })
                })
            })
            .read_providers(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_providers(
                    &settings.providers.url,
                    &settings.app_data_path.join("providers.json"),
                    Duration::from_secs(settings.providers.memory_ttl_seconds),
                );
                ports::ReadProviders::new(move || {
                    let future = read();
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_file(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_file::read_file(&settings.app_data_path);
                ports::ReadFile::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .create_process(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = create_process::create_process(&settings.app_data_path);
                ports::CreateProcess::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .update_file(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = update_file::update_file(&settings.app_data_path);
                ports::UpdateFile::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .create_file(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = create_file::create_file(&settings.app_data_path);
                ports::CreateFile::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_directory(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_directory::read_directory(&settings.app_data_path);
                ports::ReadDirectory::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_paths(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_paths::read_paths(&settings.app_data_path);
                ports::ReadPaths::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_text_matches(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_text_matches::read_text_matches(&settings.app_data_path);
                ports::ReadTextMatches::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_code_matches(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_code_matches::read_code_matches(&settings.app_data_path);
                ports::ReadCodeMatches::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_web_search(|deps| {
                let settings: &Settings = deps.as_ref();
                let implementation = read_web_search::read_web_search(&settings.app_data_path);
                ports::ReadWebSearch::new(move |workspace, arguments, cancellation| {
                    let future = implementation(workspace, arguments, cancellation);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_tools(|deps| {
                let settings: &Settings = deps.as_ref();
                let read = read_tools::read_tools(settings.tools.enabled);
                ports::ReadTools::new(move || {
                    let future = read();
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .read_tool_invocation(|_| {
                let read = read_tool_invocation::read_tool_invocation();
                ports::ReadToolInvocation::new(move |tool, args| Ok(read(tool, args)?))
            })
            .create_tool_output(|deps| {
                let settings: &Settings = deps.as_ref();
                let create = create_tool_output::create_tool_output(&settings.app_data_path);
                ports::CreateToolOutput::new(move |id, call| {
                    let future = create(id, call);
                    Box::pin(async move { Ok(future.await?) })
                })
            })
            .create_native_decision(|deps| {
                let settings: &Settings = deps.as_ref();
                let models: &ports::ReadModels = deps.as_ref();
                let policy: &ports::ReadModelAllowlist = deps.as_ref();
                let work: &ports::ReadWork = deps.as_ref();
                let conversation: &ports::ReadConversation = deps.as_ref();
                let implementation = create_native_decision::create_native_decision(
                    &settings.decider,
                    &settings.authentication.openrouter.issuer,
                    &settings.app_data_path.join("auth/openrouter.json"),
                    models.clone(),
                    policy.clone(),
                    work.clone(),
                    conversation.clone(),
                );
                ports::CreateNativeDecision::new(implementation)
            })
            .create_assignment(|deps| deps.into())
            .create_work(|deps| deps.into())
            .build()
    }
}
