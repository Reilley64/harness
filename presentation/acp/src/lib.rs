mod agents;
mod authentication;
mod conversations;
mod dependencies;
mod guidance;
mod mcp;
mod projection;
mod prompt;
mod setup;
mod steering;
mod tools;
mod usage;
mod work;

mod model_allowlist;
mod model_selection;
mod models;

mod providers;

pub use dependencies::Dependencies;

use crate::model_allowlist::ModelAllowlistResponse;
use crate::model_allowlist::ReadModelAllowlistRequest;
use crate::model_allowlist::UpdateModelAllowlistRequest;
use agent_client_protocol::Agent;
use agent_client_protocol::Error;
use agent_client_protocol::Stdio;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::v1::schema::AgentCapabilities;
use agent_client_protocol::v1::schema::AuthMethod;
use agent_client_protocol::v1::schema::AuthMethodAgent;
use agent_client_protocol::v1::schema::AuthenticateRequest;
use agent_client_protocol::v1::schema::AuthenticateResponse;
use agent_client_protocol::v1::schema::CancelNotification;
use agent_client_protocol::v1::schema::Implementation;
use agent_client_protocol::v1::schema::InitializeRequest;
use agent_client_protocol::v1::schema::InitializeResponse;
use agent_client_protocol::v1::schema::LoadSessionRequest;
use agent_client_protocol::v1::schema::LoadSessionResponse;
use agent_client_protocol::v1::schema::NewSessionRequest;
use agent_client_protocol::v1::schema::NewSessionResponse;
use agent_client_protocol::v1::schema::PromptRequest;
use agent_client_protocol::v1::schema::SessionNotification;
use application::error_context::ResultExt;
use application::ports::PresentAuthorization;
use application::use_cases::create_conversation;
use application::use_cases::create_provider_connection;
use application::use_cases::read_conversation;
use application::use_cases::read_model_allowlist;
use application::use_cases::read_model_selection;
use application::use_cases::read_models;
use application::use_cases::read_providers;
use application::use_cases::update_mcp_configuration;
use application::use_cases::update_model_allowlist;
use application::use_cases::update_model_selection;

use crate::model_selection::ModelSelectionResponse;
use crate::model_selection::ReadModelSelectionRequest;
use crate::model_selection::UpdateModelSelectionRequest;
use crate::models::ModelSummary;
use crate::models::ModelsRequest;
use crate::models::ModelsResponse;
use crate::providers::ProviderSummary;
use crate::providers::ProvidersRequest;
use crate::providers::ProvidersResponse;

pub async fn run(dependencies: impl Into<Dependencies>) -> Result<(), Error> {
    let Dependencies {
        mcp,
        providers,
        read_work,
        update_work,
        authentication,
        models,
        create_agent,
        read_agent,
        read_setup,
        update_setup,
        update_agent,
        read_guidance,
        update_guidance,
        create_conversation,
        read_conversation,
        create_message,
        read_model_selection,
        update_model_selection,
        read_model_allowlist,
        update_model_allowlist,
    } = dependencies.into();
    let sessions = conversations::Sessions::default();
    let agent_sessions = sessions.clone();
    let read_agent_sessions = sessions.clone();
    let resume_sessions = sessions.clone();
    let resume_history = read_conversation.clone();
    let resume_dependencies = create_message.clone();
    let new_sessions = sessions.clone();
    let loaded_sessions = sessions.clone();
    let prompted_sessions = sessions.clone();
    let steering_sessions = sessions.clone();
    let steering_history = read_conversation.clone();
    let projection_history = read_conversation.clone();
    Agent
        .v1()
        .name("harness")
        .on_receive_request(
            async |_: InitializeRequest, responder, _connection| {
                let metadata: serde_json::Map<String, serde_json::Value> = [
                    ("harness/mcp".into(), serde_json::json!(true)),
                    ("harness/providerCatalog".into(), serde_json::json!(true)),
                    ("harness/modelCatalog".into(), serde_json::json!(true)),
                    ("harness/modelSelection".into(), serde_json::json!(true)),
                    ("harness/modelAllowlist".into(), serde_json::json!(true)),
                    ("harness/tools".into(), serde_json::json!(true)),
                    ("harness/steering".into(), serde_json::json!(true)),
                    ("harness/usage".into(), serde_json::json!(true)),
                    ("harness/agents".into(), serde_json::json!(true)),
                    ("harness/guidance".into(), serde_json::json!(true)),
                    ("harness/approvedWork".into(), serde_json::json!(true)),
                    ("harness/setup".into(), serde_json::json!(true)),
                ]
                .into_iter()
                .collect();
                responder.respond(
                    InitializeResponse::new(ProtocolVersion::V1)
                        .agent_info(Implementation::new("harness", env!("CARGO_PKG_VERSION")))
                        .agent_capabilities(AgentCapabilities::new().load_session(true))
                        .auth_methods(vec![
                            AuthMethod::Agent(AuthMethodAgent::new("openai", "Continue with ChatGPT").description("Connect your ChatGPT plan to Harness")),
                            AuthMethod::Agent(AuthMethodAgent::new("openrouter", "Continue with OpenRouter").description("Connect a separately billed OpenRouter API key; no inference during setup")),
                        ])
                        .meta(metadata),
                )
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: work::ReadRequest, responder, connection| {
                let d=read_work.clone();
                connection.spawn(async move {
                    match application::use_cases::read_work::call(d,request.id).await {
                        Ok(o)=>responder.respond(work::Response {work:o.work.as_ref().map(work::summary),conversation_id:None}),
                        Err(e)=>responder.respond_with_error(conversations::error(e)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: work::ApproveRequest, responder, connection| {
                let d=update_work.clone();
                connection.spawn(async move {
                    let selection=domain::ModelSelection {provider_id:domain::ProviderId(request.provider_id),model_id:request.model_id,effort:request.effort.map(|e|match e {models::ModelEffort::Low=>domain::ModelEffort::Low,models::ModelEffort::Medium=>domain::ModelEffort::Medium,models::ModelEffort::High=>domain::ModelEffort::High,models::ModelEffort::ExtraHigh=>domain::ModelEffort::ExtraHigh})};
                    match application::use_cases::update_work::call(d,request.id,request.revision,request.version,request.intent,selection,request.session_revision).await {
                        Ok(o)=>responder.respond(work::Response {work:Some(work::summary(&o.work)),conversation_id:Some(o.conversation_id)}),
                        Err(e)=>responder.respond_with_error(conversations::error(e)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: mcp::Request, responder, connection| {
                let dependencies = mcp.clone();
                connection.spawn(async move {
                    let cancellation = tokio_util::sync::CancellationToken::new();
                    let operation = update_mcp_configuration::call(dependencies, request.workspace, request.operation.into(), cancellation.clone());
                    tokio::pin!(operation);
                    let request_cancellation = responder.cancellation();
                    let result = tokio::select! {
                        result = &mut operation => result,
                        _ = request_cancellation.cancelled() => { cancellation.cancel(); operation.await }
                    };
                    match result {
                        Ok(output) => responder.respond(mcp::Response::from(output.configuration)),
                        Err(error) => responder.respond_with_error(Error::new(-32006, "MCP operation failed; check configuration, trust and provisioned references. Reload to reconcile conflicting edits; custom/manual setup remains available.").data(application::error_context::diagnostic(&error))),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |_: ProvidersRequest, responder, connection| {
                let dependencies = providers.clone();
                connection.spawn(async move {
                    let result = responder
                        .cancellation()
                        .run_until_cancelled(async {
                            read_providers::call(dependencies)
                                .await
                                .context("Could not load the provider catalog")
                                .map_err(|error| {
                                    Error::new(-32001, "Could not load providers")
                                        .data(application::error_context::diagnostic(&error))
                                })
                        })
                        .await;
                    match result {
                        Ok(providers) => responder.respond(ProvidersResponse {
                            providers: providers
                                .into_iter()
                                .map(|provider| ProviderSummary {
                                    id: provider.id.0,
                                    name: provider.name,
                                })
                                .collect(),
                        }),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |_: ModelsRequest, responder, connection| {
                let dependencies = models.clone();
                connection.spawn(async move {
                    match responder
                        .cancellation()
                        .run_until_cancelled(async {
                            read_models::call(dependencies)
                                .await
                                .context("Could not load the model catalog")
                                .map_err(|error| {
                                    Error::new(-32003, "Could not load models")
                                        .data(application::error_context::diagnostic(&error))
                                })
                        })
                        .await
                    {
                        Ok(models) => responder.respond(ModelsResponse {
                            models: models.into_iter().map(ModelSummary::from).collect(),
                        }),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: AuthenticateRequest, responder, connection| {
                let authentication = authentication.clone();
                let auth_connection = connection.clone();
                connection.spawn(async move {
                    let connection = auth_connection;
                    let provider_id = request.method_id.to_string();
                    if !matches!(provider_id.as_str(), "openai" | "openrouter") {
                        return responder.respond_with_error(
                            Error::invalid_params().data("Unsupported authentication method"),
                        );
                    }
                    let authentication_id = request
                        .meta
                        .as_ref()
                        .and_then(|meta| meta.get("harness/authenticationId"))
                        .and_then(serde_json::Value::as_str)
                        .filter(|id| id.len() <= 128)
                        .map(str::to_owned);
                    let challenge_provider = provider_id.clone();
                    let present_authorization = PresentAuthorization::new(move |challenge| {
                        connection
                            .send_notification(authentication::AuthorizationChallenge {
                                authentication_id: authentication_id.clone(),
                                provider_id: challenge_provider.clone(),
                                authorization_url: challenge.authorization_url,
                            })
                            .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;
                        Ok(())
                    });
                    match responder
                        .cancellation()
                        .run_until_cancelled(async {
                            create_provider_connection::call(
                                authentication.clone(),
                                provider_id.into(),
                                present_authorization,
                            )
                            .await
                            .map_err(authentication::error)
                        })
                        .await
                    {
                        Ok(created) => responder.respond(
                            AuthenticateResponse::new().meta(
                                [(
                                    "harness/providerConnection".into(),
                                    serde_json::json!({
                                        "id": created.id,
                                        "providerId": created.provider_id.0,
                                    }),
                                )]
                                .into_iter()
                                .collect::<serde_json::Map<String, serde_json::Value>>(),
                            ),
                        ),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: NewSessionRequest, responder, connection| {
                let dependencies = create_conversation.clone();
                let sessions = new_sessions.clone();
                connection.spawn(async move {
                    let result = async {
                        let selection = conversations::selection(request.meta.as_ref())?
                            .ok_or_else(|| {
                                Error::invalid_params().data("Choose a model with /model first")
                            })?;
                        let output =
                            create_conversation::call(dependencies, selection, request.cwd.clone())
                                .await
                                .map_err(conversations::error)?;
                        sessions.register(
                            &output.conversation.id,
                            output.conversation.workspace.clone().unwrap_or(request.cwd),
                        );
                        Ok(NewSessionResponse::new(output.conversation.id.clone())
                            .meta(conversations::metadata(&output.conversation)))
                    }
                    .await;
                    match result {
                        Ok(response) => responder.respond(response),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: LoadSessionRequest, responder, connection| {
                let dependencies = read_conversation.clone();
                let sessions = loaded_sessions.clone();
                let updates = connection.clone();
                connection.spawn(async move {
                    let result = async {
                        let output =
                            read_conversation::call(dependencies, request.session_id.to_string())
                                .await
                                .map_err(conversations::error)?;
                        sessions.register(
                            &output.conversation.id,
                            output.conversation.workspace.clone().unwrap_or(request.cwd),
                        );
                        let mut projection =
                            projection::Projection::new(Some(&output.conversation.model));
                        for update in projection.replay(&output.conversation) {
                            updates.send_notification(SessionNotification::new(
                                output.conversation.id.clone(),
                                update,
                            ))?;
                        }
                        updates.send_notification(usage::Notification {
                            session_id: output.conversation.id.clone(),
                            usage: output.conversation.usage().into(),
                        })?;
                        Ok(LoadSessionResponse::new()
                            .meta(conversations::metadata(&output.conversation)))
                    }
                    .await;
                    match result {
                        Ok(response) => responder.respond(response),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: PromptRequest, responder, connection| {
                prompt::receive(
                    request,
                    responder,
                    connection,
                    create_message.clone(),
                    projection_history.clone(),
                    prompted_sessions.clone(),
                    false,
                )
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: steering::Request, responder, connection| {
                if request.text.trim().is_empty()
                    || request.text.len() > 1_048_576
                    || !steering::valid_id(&request.id)
                {
                    return responder.respond(steering::Response { accepted: false });
                }
                let sender = steering_sessions.steering(&request.session_id);
                let history = steering_history.clone();
                connection.spawn(async move {
                    let mut accepted = false;
                    if let Some(sender) = sender {
                        let (acknowledgement, result) = tokio::sync::oneshot::channel();
                        let instruction = application::turn_control::SteeringRequest {
                            id: request.id.clone(),
                            text: request.text.clone(),
                            accepted: acknowledgement,
                        };
                        let outcome = async {
                            sender.try_send(instruction).ok()?;
                            result.await.ok()
                        };
                        accepted =
                            tokio::time::timeout(std::time::Duration::from_secs(30), outcome)
                                .await
                                .ok()
                                .flatten()
                                .unwrap_or(false);
                    }
                    if !accepted {
                        accepted = read_conversation::call(history, request.session_id)
                            .await
                            .ok()
                            .is_some_and(|output| {
                                output
                                    .conversation
                                    .steering_accepted(&request.id, &request.text)
                                    .unwrap_or(false)
                            });
                    }
                    responder.respond(steering::Response { accepted })
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_notification(
            async move |notification: CancelNotification, _connection| {
                sessions.cancel(&notification.session_id.to_string());
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |_: ReadModelAllowlistRequest, responder, connection| {
                let dependencies = read_model_allowlist.clone();
                connection.spawn(async move {
                    match read_model_allowlist::call(dependencies).await {
                        Ok(output) => {
                            responder.respond(ModelAllowlistResponse::from(output.policy))
                        }
                        Err(error) => responder.respond_with_error(model_allowlist::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: UpdateModelAllowlistRequest, responder, connection| {
                let dependencies = update_model_allowlist.clone();
                connection.spawn(async move {
                    match update_model_allowlist::call(dependencies, request.patterns).await {
                        Ok(output) => {
                            responder.respond(ModelAllowlistResponse::from(output.policy))
                        }
                        Err(error) => responder.respond_with_error(model_allowlist::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |_: ReadModelSelectionRequest, responder, connection| {
                let dependencies = read_model_selection.clone();
                connection.spawn(async move {
                    match read_model_selection::call(dependencies).await {
                        Ok(output) => responder.respond(ModelSelectionResponse {
                            selection: output.selected_model.map(Into::into),
                        }),
                        Err(error) => responder.respond_with_error(model_selection::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: UpdateModelSelectionRequest, responder, connection| {
                let dependencies = update_model_selection.clone();
                connection.spawn(async move {
                    match update_model_selection::call(dependencies, request.into()).await {
                        Ok(output) => responder.respond(ModelSelectionResponse {
                            selection: Some(output.selected_model.into()),
                        }),
                        Err(error) => responder.respond_with_error(model_selection::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: agents::ReadRequest, responder, connection| {
                let dependencies = read_agent.clone();
                let sessions = read_agent_sessions.clone();
                connection.spawn(async move {
                    let role = match setup::role(request.role) {
                        Ok(role) => role,
                        Err(error) => {
                            return responder
                                .respond_with_error(Error::invalid_params().data(error));
                        }
                    };
                    match responder
                        .cancellation()
                        .run_until_cancelled(async {
                            application::use_cases::read_agent::call(
                                dependencies,
                                request.cwd,
                                role,
                            )
                            .await
                            .map_err(conversations::error)
                        })
                        .await
                    {
                        Ok(output) => {
                            if let Some(conversation) = output.conversation {
                                sessions.register(
                                    &conversation.id,
                                    conversation
                                        .workspace
                                        .clone()
                                        .expect("Griller has a workspace"),
                                );
                                responder.respond(agents::response(&conversation))
                            } else {
                                responder.respond(agents::Response { conversation: None })
                            }
                        }
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: agents::StartRequest, responder, connection| {
                let dependencies = create_agent.clone();
                let sessions = agent_sessions.clone();
                connection.spawn(async move {
                    let result = async {
                        let selection = conversations::selection(Some(&request.selection))?
                            .ok_or_else(|| Error::invalid_params().data("Choose a model first"))?;
                        let output = application::use_cases::create_agent::call(
                            dependencies,
                            selection,
                            request.cwd,
                            setup::role(request.role)
                                .map_err(|error| Error::invalid_params().data(error))?,
                        )
                        .await
                        .map_err(conversations::error)?;
                        sessions.register(
                            &output.conversation.id,
                            output
                                .conversation
                                .workspace
                                .clone()
                                .expect("Griller has a workspace"),
                        );
                        Ok(agents::response(&output.conversation))
                    }
                    .await;
                    match result {
                        Ok(response) => responder.respond(response),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: agents::ResumeRequest, responder, connection| {
                let request = PromptRequest::new(
                    request.session_id,
                    vec![agent_client_protocol::v1::schema::ContentBlock::Text(
                        agent_client_protocol::v1::schema::TextContent::new("Resume"),
                    )],
                );
                prompt::receive(
                    request,
                    responder,
                    connection,
                    resume_dependencies.clone(),
                    resume_history.clone(),
                    resume_sessions.clone(),
                    true,
                )
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: agents::AcknowledgeRequest, responder, connection| {
                let dependencies = update_agent.clone();
                connection.spawn(async move {
                    match application::use_cases::update_agent::call(
                        dependencies,
                        request.session_id,
                    )
                    .await
                    {
                        Ok(output) => responder.respond(agents::response(&output.conversation)),
                        Err(error) => responder.respond_with_error(conversations::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: setup::ReadRequest, responder, connection| {
                let dependencies = read_setup.clone();
                connection.spawn(async move {
                    let result = responder
                        .cancellation()
                        .run_until_cancelled(async {
                            application::use_cases::read_setup::call(
                                dependencies,
                                request.cwd,
                                request.session_id,
                                tokio_util::sync::CancellationToken::new(),
                            )
                            .await
                            .map(setup::Response::from)
                            .map_err(conversations::error)
                        })
                        .await;
                    match result {
                        Ok(output) => responder.respond(output),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: setup::UpdateRequest, responder, connection| {
                let dependencies = update_setup.clone();
                connection.spawn(async move {
                    match application::use_cases::update_setup::call(
                        dependencies,
                        request.session_id,
                        request.revision,
                        request.proposal,
                        request.confirm,
                        tokio_util::sync::CancellationToken::new(),
                    )
                    .await
                    {
                        Ok(output) => responder.respond(agents::response(&output.conversation)),
                        Err(error) => responder.respond_with_error(conversations::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: guidance::ReadRequest, responder, connection| {
                let dependencies = read_guidance.clone();
                connection.spawn(async move {
                    match application::use_cases::read_guidance::call(
                        dependencies,
                        request.session_id,
                        request.cwd,
                        request.phase,
                        request.scopes,
                        request.references,
                    )
                    .await
                    {
                        Ok(output) => responder.respond(guidance::Response::from(output)),
                        Err(error) => responder.respond_with_error(conversations::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: guidance::AdoptRequest, responder, connection| {
                let dependencies = update_guidance.clone();
                connection.spawn(async move {
                    match application::use_cases::update_guidance::call(
                        dependencies,
                        request.session_id,
                        request.revision,
                        request.available,
                        request.phase,
                        request.scopes,
                        request.references,
                    )
                    .await
                    {
                        Ok(output) => responder.respond(agents::response(&output.conversation)),
                        Err(error) => responder.respond_with_error(conversations::error(error)),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_to(Stdio::new())
        .await
}
