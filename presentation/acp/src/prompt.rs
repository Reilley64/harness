use crate::conversations;
use crate::projection;
use crate::usage;
use agent_client_protocol::Client;
use agent_client_protocol::Error;
use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::Responder;
use agent_client_protocol::v1::schema::PromptRequest;
use agent_client_protocol::v1::schema::PromptResponse;
use agent_client_protocol::v1::schema::SessionNotification;
use agent_client_protocol::v1::schema::StopReason;
use application::ports::PresentTurnEvent;
use application::use_cases::create_message;
use application::use_cases::read_conversation;
use application::use_cases::update_run;

pub(crate) fn receive(
    request: PromptRequest,
    responder: Responder<PromptResponse>,
    connection: ConnectionTo<Client>,
    dependencies: create_message::Dependencies,
    history: read_conversation::Dependencies,
    sessions: conversations::Sessions,
    resume: bool,
) -> Result<(), Error> {
    let selection = match conversations::selection(request.meta.as_ref()) {
        Ok(selection) => selection,
        Err(error) => return responder.respond_with_error(error),
    };
    let text = match projection::prompt(request.prompt) {
        Ok(text) => text,
        Err(error) => return responder.respond_with_error(error),
    };
    let mut running = match sessions.start(&request.session_id.to_string()) {
        Ok(running) => running,
        Err(error) => return responder.respond_with_error(error),
    };
    let dependencies = dependencies.clone();
    let history = history.clone();
    let updates = connection.clone();
    let final_updates = connection.clone();
    connection.spawn(async move {
        let id = request.session_id.to_string();
        let model = read_conversation::call(history, id.clone())
            .await
            .ok()
            .map(|output| output.conversation.model)
            .filter(|model| {
                selection.as_ref().is_none_or(|selection| {
                    selection.model_id == model.id && selection.provider_id == model.provider_id
                })
            });
        let projection = parking_lot::Mutex::new(projection::Projection::new(model.as_ref()));
        let updated_session = id.clone();
        let present = PresentTurnEvent::new(move |event| {
            if let domain::TurnEvent::Usage { usage } = &event {
                updates
                    .send_notification(usage::Notification {
                        session_id: updated_session.clone(),
                        usage: usage.clone().into(),
                    })
                    .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;
            }
            for update in projection.lock().event(event) {
                updates
                    .send_notification(SessionNotification::new(updated_session.clone(), update))
                    .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;
            }
            Ok(())
        });
        let result = async {
            let steering = running.steering.take().expect("New reply owns steering");
            if resume {
                update_run::call(
                    dependencies,
                    id,
                    present,
                    steering,
                    running.workspace.clone(),
                    running.cancellation.clone(),
                )
                .await
            } else {
                create_message::call(
                    dependencies,
                    id,
                    text,
                    selection,
                    present,
                    steering,
                    running.workspace.clone(),
                    running.cancellation.clone(),
                )
                .await
            }
        };
        tokio::pin!(result);
        let request_cancellation = responder.cancellation();
        let result = tokio::select! {
            result = &mut result => result,
            _ = request_cancellation.cancelled() => {
                running.cancellation.cancel();
                result.await
            }
        };
        match result {
            Ok(output) => {
                let mut projection = projection::Projection::new(Some(&output.conversation.model));
                for update in projection.event(domain::TurnEvent::Usage {
                    usage: output.conversation.usage(),
                }) {
                    if let Err(error) = final_updates.send_notification(SessionNotification::new(
                        output.conversation.id.clone(),
                        update,
                    )) {
                        return responder.respond_with_error(error);
                    }
                }
                let stop_reason = if running.cancellation.is_cancelled() {
                    StopReason::Cancelled
                } else {
                    match output
                        .conversation
                        .messages
                        .last()
                        .map(|message| message.status)
                    {
                        Some(domain::MessageStatus::Cancelled) => StopReason::Cancelled,
                        Some(domain::MessageStatus::Refused) => StopReason::Refusal,
                        Some(domain::MessageStatus::Limited) => StopReason::MaxTurnRequests,
                        Some(domain::MessageStatus::Incomplete) => StopReason::MaxTokens,
                        _ => StopReason::EndTurn,
                    }
                };
                responder.respond(
                    PromptResponse::new(stop_reason)
                        .meta(conversations::metadata(&output.conversation)),
                )
            }
            Err(error) => responder.respond_with_error(conversations::error(error)),
        }
    })
}
