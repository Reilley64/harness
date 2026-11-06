use crate::MAX_MODEL_STEPS;
use crate::Message;
use crate::MessageRole;
use crate::MessageStatus;
use crate::Model;
use crate::ModelEffort;
use crate::ModelStep;
use crate::SteeringInstruction;
use crate::ToolCall;
use crate::UsageSummary;

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub provider_connection_id: String,
    pub workspace: Option<std::path::PathBuf>,
    pub model: Model,
    pub effort: Option<ModelEffort>,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
    pub revision: u64,
}

impl Conversation {
    pub fn usage(&self) -> UsageSummary {
        let mut input = 0u64;
        let mut output = 0u64;
        let mut known = false;
        let mut partial = false;
        let mut context_tokens = None;
        let mut model_id = None;
        for message in self
            .messages
            .iter()
            .filter(|message| message.role == MessageRole::Assistant)
        {
            let reports;
            let usage = if let Some(usage) = &message.generation_usage {
                usage.as_slice()
            } else {
                partial |= !message.steering.is_empty()
                    || (message.steps.is_empty() && message.status != MessageStatus::InProgress)
                    || matches!(
                        message.status,
                        MessageStatus::Cancelled
                            | MessageStatus::Interrupted
                            | MessageStatus::Failed
                    );
                reports = message
                    .steps
                    .iter()
                    .map(|step| step.usage)
                    .collect::<Vec<_>>();
                reports.as_slice()
            };
            for report in usage {
                model_id = Some(message.model_id.clone());
                context_tokens =
                    report.map(|usage| usage.input_tokens.saturating_add(usage.output_tokens));
                if let Some(usage) = report {
                    known = true;
                    input = input.saturating_add(usage.input_tokens);
                    output = output.saturating_add(usage.output_tokens);
                } else {
                    partial = true;
                }
            }
        }
        UsageSummary {
            input_tokens: (known || !partial).then_some(input),
            output_tokens: (known || !partial).then_some(output),
            context_tokens,
            model_id,
            partial,
        }
    }

    pub fn begin_message(
        &mut self,
        user_id: String,
        reply_id: String,
        content: String,
    ) -> Result<(), &'static str> {
        if content.trim().is_empty() {
            return Err("A message cannot be empty");
        }
        if self
            .messages
            .iter()
            .any(|message| message.status == MessageStatus::InProgress)
        {
            return Err("A reply is already in progress");
        }
        for (id, role, content, status) in [
            (
                user_id,
                MessageRole::User,
                content,
                MessageStatus::Completed,
            ),
            (
                reply_id,
                MessageRole::Assistant,
                String::new(),
                MessageStatus::InProgress,
            ),
        ] {
            self.messages.push(Message {
                id,
                role,
                content,
                status,
                model_id: self.model.id.clone(),
                effort: self.effort,
                steps: Vec::new(),
                steering: Vec::new(),
                generation_usage: (role == MessageRole::Assistant).then(Vec::new),
            });
        }
        Ok(())
    }

    pub fn finish_reply(
        &mut self,
        content: String,
        status: MessageStatus,
    ) -> Result<(), &'static str> {
        if status == MessageStatus::InProgress {
            return Err("A reply must finish with a terminal status");
        }
        let reply = self.active_reply_mut()?;
        for call in reply.steps.iter_mut().flat_map(|step| &mut step.calls) {
            call.stop();
        }
        reply.content = content;
        reply.status = status;
        Ok(())
    }

    pub fn active_reply(&self) -> Result<&Message, &'static str> {
        self.messages
            .last()
            .filter(|reply| {
                reply.role == MessageRole::Assistant && reply.status == MessageStatus::InProgress
            })
            .ok_or("There is no reply in progress")
    }

    pub fn active_reply_mut(&mut self) -> Result<&mut Message, &'static str> {
        self.messages
            .last_mut()
            .filter(|reply| {
                reply.role == MessageRole::Assistant && reply.status == MessageStatus::InProgress
            })
            .ok_or("There is no reply in progress")
    }

    pub fn record_step(
        &mut self,
        step: ModelStep,
        content: String,
    ) -> Result<Vec<ToolCall>, &'static str> {
        let mut ids = std::collections::HashSet::new();
        let mut tool_ids = std::collections::HashSet::new();
        if self
            .messages
            .iter()
            .flat_map(|message| &message.steps)
            .flat_map(|step| &step.calls)
            .chain(&step.calls)
            .any(|call| !ids.insert(&call.call_id) || !tool_ids.insert(&call.id))
        {
            return Err("A tool-call identity was reused; nothing was executed");
        }
        let reply = self.active_reply_mut()?;
        if reply.steps.len() >= MAX_MODEL_STEPS
            || reply
                .steps
                .iter()
                .map(|step| step.output.len())
                .sum::<usize>()
                + step.output.len()
                > 4_194_304
        {
            return Err("Turn continuation exceeds its limits");
        }
        let calls = step.calls.clone();
        reply.content = content;
        reply.steps.push(step);
        Ok(calls)
    }

    pub fn update_tool_call(&mut self, call: ToolCall) -> Result<(), &'static str> {
        let recorded = self
            .active_reply_mut()?
            .steps
            .iter_mut()
            .flat_map(|step| &mut step.calls)
            .find(|recorded| recorded.id == call.id)
            .ok_or("Tool call is not part of the active reply")?;
        if recorded.call_id != call.call_id
            || recorded.name != call.name
            || recorded.arguments != call.arguments
            || recorded.text_offset != call.text_offset
        {
            return Err("Tool call identity cannot change during execution");
        }
        if !recorded.permits(call.status) {
            return Err("Invalid recorded tool transition");
        }
        *recorded = call;
        Ok(())
    }

    pub fn steering_accepted(&self, id: &str, text: &str) -> Result<bool, &'static str> {
        if let Some(instruction) = self
            .messages
            .iter()
            .flat_map(|message| &message.steering)
            .find(|instruction| instruction.id == id)
        {
            return if instruction.text == text {
                Ok(true)
            } else {
                Err("Steering identity is already bound to different text")
            };
        }
        Ok(false)
    }

    pub fn steer(
        &mut self,
        id: String,
        text: String,
        content: String,
    ) -> Result<bool, &'static str> {
        if self.steering_accepted(&id, &text)? {
            return Ok(false);
        }
        let reply = self.active_reply_mut()?;
        if text.trim().is_empty()
            || reply.steering.len() >= 64
            || reply
                .steering
                .iter()
                .map(|instruction| instruction.text.len())
                .sum::<usize>()
                + text.len()
                > 1_048_576
        {
            return Err("Steering instruction exceeds its limits");
        }
        reply.content = content;
        reply.steering.push(SteeringInstruction {
            id,
            text,
            after_step: reply.steps.len(),
        });
        Ok(true)
    }

    pub fn history(&self) -> Vec<&Message> {
        self.messages
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|pair| match pair[1].status {
                MessageStatus::Completed => vec![&pair[0], &pair[1]],
                MessageStatus::InProgress => vec![&pair[0]],
                _ => Vec::new(),
            })
            .collect()
    }

    pub fn recover(&mut self) -> bool {
        let mut changed = false;
        for message in &mut self.messages {
            for step in &mut message.steps {
                for call in &mut step.calls {
                    changed |= call.recover();
                }
            }
            if message.status == MessageStatus::InProgress {
                message.status = MessageStatus::Interrupted;
                changed = true;
            }
        }
        changed
    }
}
