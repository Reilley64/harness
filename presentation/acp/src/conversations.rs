use crate::models::ModelEffort;
use application::error_context::ResultExt;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use agent_client_protocol::Error;
use agent_client_protocol::v1::schema::Meta;
use tokio_util::sync::CancellationToken;

use crate::models::ModelSummary;
use crate::provider_retries;
use crate::tools;

#[derive(Clone, Default)]
pub(crate) struct Sessions(Arc<Mutex<HashMap<String, Session>>>);
struct Session {
    workspace: std::path::PathBuf,
    active: Option<Active>,
}
struct Active {
    cancellation: CancellationToken,
    steering: tokio::sync::mpsc::Sender<application::turn_control::SteeringRequest>,
}
pub(crate) struct RunningReply {
    pub cancellation: CancellationToken,
    pub steering: Option<tokio::sync::mpsc::Receiver<application::turn_control::SteeringRequest>>,
    pub workspace: std::path::PathBuf,
    sessions: Sessions,
    id: String,
}
impl Drop for RunningReply {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(session) = self.sessions.0.lock().get_mut(&self.id) {
            session.active = None;
        }
    }
}
impl Sessions {
    pub fn register(&self, id: &str, workspace: std::path::PathBuf) {
        self.0.lock().entry(id.to_owned()).or_insert(Session {
            workspace,
            active: None,
        });
    }
    pub fn start(&self, id: &str) -> Result<RunningReply, Error> {
        let mut sessions = self.0.lock();
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| Error::invalid_params().data("Create or load this session first"))?;
        if session.active.is_some() {
            return Err(Error::new(-32004, "A reply is already in progress"));
        }
        let cancellation = CancellationToken::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        session.active = Some(Active {
            cancellation: cancellation.clone(),
            steering: sender,
        });
        Ok(RunningReply {
            cancellation,
            steering: Some(receiver),
            workspace: session.workspace.clone(),
            sessions: self.clone(),
            id: id.to_owned(),
        })
    }
    pub fn cancel(&self, id: &str) {
        if let Some(active) = self.0.lock().get(id).and_then(|s| s.active.as_ref()) {
            active.cancellation.cancel();
        }
    }
    pub fn steering(
        &self,
        id: &str,
    ) -> Option<tokio::sync::mpsc::Sender<application::turn_control::SteeringRequest>> {
        self.0
            .lock()
            .get(id)
            .and_then(|s| s.active.as_ref())
            .map(|a| a.steering.clone())
    }
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Selection {
    provider_id: String,
    model_id: String,
    effort: Option<ModelEffort>,
}

pub(crate) fn selection(meta: Option<&Meta>) -> Result<Option<domain::ModelSelection>, Error> {
    let Some(value) = meta.and_then(|meta| meta.get("harness/modelSelection")) else {
        return Ok(None);
    };
    let selection: Selection = serde_json::from_value(value.clone())
        .context("Invalid model selection")
        .map_err(|error| {
            Error::invalid_params().data(application::error_context::diagnostic(&error))
        })?;
    Ok(Some(domain::ModelSelection {
        provider_id: selection.provider_id.into(),
        model_id: selection.model_id,
        effort: selection.effort.map(|effort| match effort {
            ModelEffort::Low => domain::ModelEffort::Low,
            ModelEffort::Medium => domain::ModelEffort::Medium,
            ModelEffort::High => domain::ModelEffort::High,
            ModelEffort::ExtraHigh => domain::ModelEffort::ExtraHigh,
        }),
    }))
}

pub(crate) fn metadata(conversation: &domain::Conversation) -> Meta {
    let model = ModelSummary::from(application::use_cases::read_models::Output {
        blocked: None,
        id: conversation.model.id.clone(),
        name: conversation.model.name.clone(),
        context_window: conversation.model.context_window,
        provider_id: conversation.model.provider_id.clone(),
        capabilities: conversation.model.capabilities.clone(),
        efforts: conversation.model.efforts.clone(),
        pricing: conversation.model.pricing.clone(),
    });
    [("harness/conversation".into(), serde_json::json!({
        "id": conversation.id,
        "workspace": conversation.workspace,
        "resumeRequired": conversation.resume_required,
        "providerWait": conversation.provider_retry.as_ref().map(provider_retries::summary),
        "selectedInstructions": conversation.selected_instructions,
        "assignment": conversation.assignment.as_ref().map(|a| serde_json::json!({"workId":a.work_id,"intent":a.intent,"version":a.proposal.version,"creatorId":a.creator_id,"approvedAt":a.approved_at,"document":a.proposal.document})),
        "research": conversation.research.as_ref().map(|research| serde_json::json!({"parentId":research.parent_id,"parentAgentId":research.parent_agent_id,"intent":research.intent,"assignmentId":research.assignment_id,"executionId":research.execution_id})),
        "agent": conversation.agent.as_ref().map(|agent| serde_json::json!({
            "id": agent.id.0, "role": match agent.role { domain::AgentRole::ChiefOfStaff => "chief_of_staff", domain::AgentRole::Setup => "setup", domain::AgentRole::Research => "research", _ => "griller" }, "creatorId": conversation.research.as_ref().map(|research| &research.parent_agent_id).or_else(|| conversation.assignment.as_ref().map(|assignment| &assignment.creator_id)),
        })),
        "runs": conversation.runs.iter().map(|run| serde_json::json!({
            "id": run.id, "agentId": run.agent_id.0, "replyId": run.reply_id,
            "modelId": run.model_id, "providerId": run.provider_id.0, "effort": effort(run.effort),
            "task": run.instructions.as_ref().and_then(|id| conversation.instruction_snapshots.iter().find(|snapshot| &snapshot.id == id)).map_or("grilling", |snapshot| snapshot.phase.as_str()),
            "instructions": run.instructions,
            "status": match run.status {
                domain::RunStatus::Running => "running",
                domain::RunStatus::NeedsHuman => "needs_human",
                domain::RunStatus::Stopped => "stopped",
                domain::RunStatus::Failed => "failed",
                domain::RunStatus::Interrupted => "interrupted",
            },
        })).collect::<Vec<_>>(),
        "usage": super::usage::Summary::from(conversation.usage()),
        "providerConnectionId": conversation.provider_connection_id,
        "model": model,
        "effort": effort(conversation.effort),
        "createdAt": conversation.created_at,
        "updatedAt": conversation.updated_at,
        "revision": conversation.revision,
        "messages": conversation.messages.iter().map(|message| serde_json::json!({
            "id": message.id,
            "role": match message.role { domain::MessageRole::User => "user", domain::MessageRole::Assistant => "assistant" },
            "content": message.content,
            "status": match message.status {
                domain::MessageStatus::InProgress => "in_progress",
                domain::MessageStatus::Completed => "completed",
                domain::MessageStatus::Cancelled => "cancelled",
                domain::MessageStatus::Failed => "failed",
                domain::MessageStatus::Interrupted => "interrupted",
                domain::MessageStatus::Incomplete => "incomplete",
                domain::MessageStatus::Limited => "limited",
                domain::MessageStatus::Refused => "refused",
            },
            "modelId": message.model_id, "providerId": message.provider_id.0,
            "tools": message.steps.iter().flat_map(|step|&step.calls).map(|call|tools::summary(&message.id,call)).collect::<Vec<_>>(),
            "steering": message.steering.iter().map(|s|serde_json::json!({"id":s.id,"text":s.text,"afterStep":s.after_step})).collect::<Vec<_>>(),
            "effort": effort(message.effort),
        })).collect::<Vec<_>>(),
    }))].into_iter().collect()
}

fn effort(effort: Option<domain::ModelEffort>) -> Option<&'static str> {
    effort.map(|effort| match effort {
        domain::ModelEffort::Low => "low",
        domain::ModelEffort::Medium => "medium",
        domain::ModelEffort::High => "high",
        domain::ModelEffort::ExtraHigh => "extra_high",
    })
}

pub(crate) fn error(error: impl std::error::Error + 'static) -> Error {
    let diagnostic = application::error_context::diagnostic(&error);
    Error::new(-32004, "Conversation request failed").data(diagnostic)
}
