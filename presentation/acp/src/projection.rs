use agent_client_protocol::Error;
use agent_client_protocol::v1::schema::Content;
use agent_client_protocol::v1::schema::ContentBlock;
use agent_client_protocol::v1::schema::ContentChunk;
use agent_client_protocol::v1::schema::MessageId;
use agent_client_protocol::v1::schema::SessionUpdate;
use agent_client_protocol::v1::schema::TextContent;
use agent_client_protocol::v1::schema::ToolCall;
use agent_client_protocol::v1::schema::ToolCallContent;
use agent_client_protocol::v1::schema::ToolCallStatus;
use agent_client_protocol::v1::schema::ToolCallUpdate;
use agent_client_protocol::v1::schema::ToolCallUpdateFields;
use agent_client_protocol::v1::schema::UsageUpdate;
use std::collections::HashSet;

use crate::tools;
use crate::usage;

pub(crate) fn prompt(blocks: Vec<ContentBlock>) -> Result<String, Error> {
    let mut text = String::new();
    for block in blocks {
        match block {
            ContentBlock::Text(block) => text.push_str(&block.text),
            ContentBlock::ResourceLink(link) => {
                let reference = serde_json::json!({
                    "type": "resource_link",
                    "contentsRead": false,
                    "uri": link.uri,
                    "name": link.name,
                    "description": link.description,
                    "title": link.title,
                    "mimeType": link.mime_type,
                    "size": link.size,
                });
                text.push_str("\n[Resource reference (contents not read): ");
                text.push_str(&reference.to_string());
                text.push_str("]\n");
            }
            _ => {
                return Err(Error::invalid_params()
                    .data("Harness accepts only text and resource references"));
            }
        }
        if text.len() > 1_048_576 {
            return Err(Error::invalid_params().data("Message exceeds the 1 MiB limit"));
        }
    }
    if text.trim().is_empty() {
        return Err(Error::invalid_params().data("A message cannot be empty"));
    }
    Ok(text)
}

#[derive(Default)]
pub(crate) struct Projection {
    tools: HashSet<String>,
    model: Option<(String, Option<u64>)>,
}

impl Projection {
    pub fn new(model: Option<&domain::Model>) -> Self {
        Self {
            tools: HashSet::new(),
            model: model.map(|model| (model.id.clone(), model.context_window)),
        }
    }

    pub fn event(&mut self, event: domain::TurnEvent) -> Vec<SessionUpdate> {
        match event {
            domain::TurnEvent::Text { message_id, delta } => {
                vec![text(&message_id, &delta, domain::MessageRole::Assistant)]
            }
            domain::TurnEvent::Tool { message_id, call } => self.tool(&message_id, &call),
            domain::TurnEvent::Usage { usage } => self.usage(&usage).into_iter().collect(),
        }
    }

    pub fn replay(&mut self, conversation: &domain::Conversation) -> Vec<SessionUpdate> {
        let mut updates = Vec::new();
        for message in &conversation.messages {
            if message.role == domain::MessageRole::User {
                updates.push(text(&message.id, &message.content, message.role));
                continue;
            }
            let mut calls = message
                .steps
                .iter()
                .flat_map(|step| &step.calls)
                .collect::<Vec<_>>();
            calls.sort_by_key(|call| call.text_offset);
            let mut offset = 0;
            for call in calls {
                let mut end = call.text_offset.min(message.content.len()).max(offset);
                while !message.content.is_char_boundary(end) {
                    end -= 1;
                }
                if end > offset {
                    updates.push(text(
                        &message.id,
                        &message.content[offset..end],
                        message.role,
                    ));
                }
                offset = end;
                updates.extend(self.tool(&message.id, call));
            }
            if offset < message.content.len() || message.content.is_empty() {
                updates.push(text(&message.id, &message.content[offset..], message.role));
            }
        }
        updates.extend(self.usage(&conversation.usage()));
        updates
    }

    fn tool(&mut self, message_id: &str, call: &domain::ToolCall) -> Vec<SessionUpdate> {
        let status = match call.status {
            domain::ToolStatus::Requested => ToolCallStatus::Pending,
            domain::ToolStatus::Running => ToolCallStatus::InProgress,
            domain::ToolStatus::Succeeded => ToolCallStatus::Completed,
            _ => ToolCallStatus::Failed,
        };
        let meta = [("harness/tool".into(), tools::summary(message_id, call))]
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>();
        let content = if call.output.is_empty() {
            Vec::new()
        } else {
            vec![ToolCallContent::Content(Content::new(ContentBlock::Text(
                TextContent::new(&call.output),
            )))]
        };
        let input = serde_json::from_str::<serde_json::Value>(&call.arguments)
            .unwrap_or_else(|_| serde_json::Value::String(call.arguments.clone()));
        let mut updates = Vec::new();
        if self.tools.insert(call.id.clone()) {
            updates.push(SessionUpdate::ToolCall(
                ToolCall::new(call.id.clone(), call.name.clone())
                    .name(call.name.clone())
                    .status(status)
                    .raw_input(input.clone())
                    .content(content.clone())
                    .meta(meta.clone()),
            ));
        }
        updates.push(SessionUpdate::ToolCallUpdate(
            ToolCallUpdate::new(
                call.id.clone(),
                ToolCallUpdateFields::new()
                    .title(call.name.clone())
                    .name(call.name.clone())
                    .status(status)
                    .raw_input(input)
                    .content(content),
            )
            .meta(meta),
        ));
        updates
    }

    fn usage(&self, summary: &domain::UsageSummary) -> Option<SessionUpdate> {
        let (model_id, capacity) = self.model.as_ref()?;
        if summary.model_id.as_ref()? != model_id {
            return None;
        }
        Some(SessionUpdate::UsageUpdate(
            UsageUpdate::new(summary.context_tokens?, (*capacity)?).meta(
                [(
                    "harness/usage".into(),
                    serde_json::json!(usage::Summary::from(summary.clone())),
                )]
                .into_iter()
                .collect::<serde_json::Map<String, serde_json::Value>>(),
            ),
        ))
    }
}

fn text(message_id: &str, value: &str, role: domain::MessageRole) -> SessionUpdate {
    let chunk = ContentChunk::new(ContentBlock::Text(TextContent::new(value)))
        .message_id(MessageId::new(message_id.to_owned()));
    match role {
        domain::MessageRole::User => SessionUpdate::UserMessageChunk(chunk),
        domain::MessageRole::Assistant => SessionUpdate::AgentMessageChunk(chunk),
    }
}
