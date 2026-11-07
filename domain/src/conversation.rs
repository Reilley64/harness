use crate::Agent;
use crate::InstructionSnapshot;
use crate::MAX_MODEL_STEPS;
use crate::Message;
use crate::MessageRole;
use crate::MessageStatus;
use crate::Model;
use crate::ModelEffort;
use crate::ModelStep;
use crate::Run;
use crate::RunStatus;
use crate::SteeringInstruction;
use crate::ToolCall;
use crate::ToolStatus;
use crate::UsageSummary;

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub agent: Option<Agent>,
    pub runs: Vec<Run>,
    pub instruction_snapshots: Vec<InstructionSnapshot>,
    pub selected_instructions: Option<String>,
    pub resume_required: bool,
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
    pub fn selected_instructions(&self) -> Option<&InstructionSnapshot> {
        self.selected_instructions.as_ref().and_then(|id| {
            self.instruction_snapshots
                .iter()
                .find(|snapshot| &snapshot.id == id)
        })
    }

    pub fn run_instructions(&self) -> Result<&InstructionSnapshot, &'static str> {
        let id = self.runs.last().and_then(|run| run.instructions.as_ref())
            .ok_or("Required instruction snapshot is unavailable for this run; inspect guidance, acknowledge the interruption and explicitly adopt instructions for new work")?;
        self.instruction_snapshots
            .iter()
            .find(|snapshot| &snapshot.id == id)
            .ok_or("Required run instruction snapshot is missing")
    }

    pub fn adopt_instructions(
        &mut self,
        snapshot: InstructionSnapshot,
    ) -> Result<(), &'static str> {
        if self.resume_required
            || self
                .messages
                .iter()
                .any(|message| message.status == MessageStatus::InProgress)
        {
            return Err(
                "Instruction adoption requires idle work; Resume retains its captured snapshot",
            );
        }
        snapshot.validate()?;
        if let Some(existing) = self
            .instruction_snapshots
            .iter()
            .find(|existing| existing.id == snapshot.id)
        {
            if existing != &snapshot {
                return Err("Instruction snapshot identity is already bound to different content");
            }
        } else {
            if self.instruction_snapshots.len() >= 128 {
                return Err("Instruction snapshot history is full");
            }
            self.instruction_snapshots.push(snapshot.clone());
        }
        self.selected_instructions = Some(snapshot.id);
        Ok(())
    }

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
        if self.resume_required {
            return Err("Interrupted work requires explicit Resume before another prompt");
        }
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
                provider_id: self.model.provider_id.clone(),
                effort: self.effort,
                steps: Vec::new(),
                steering: Vec::new(),
                generation_usage: (role == MessageRole::Assistant).then(Vec::new),
            });
        }
        Ok(())
    }

    pub fn begin_run(&mut self, id: String) -> Result<(), &'static str> {
        if let Some(agent) = &self.agent {
            self.selected_instructions()
                .ok_or("Required Griller instructions are not selected")?
                .validate()?;
            if self
                .runs
                .iter()
                .any(|run| run.id == id || run.status == RunStatus::Running)
            {
                return Err("An agent can have only one controlling run with a fresh identity");
            }
            let reply = self.active_reply()?;
            self.runs.push(Run {
                id,
                agent_id: agent.id.clone(),
                reply_id: reply.id.clone(),
                model_id: self.model.id.clone(),
                provider_id: self.model.provider_id.clone(),
                effort: self.effort,
                status: RunStatus::Running,
                instructions: self.selected_instructions.clone(),
            });
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        if !self.resume_required {
            return Err("There is no interrupted work to Resume");
        }
        if self.agent.is_some() {
            self.run_instructions()?.validate()?;
        }
        let reply = self
            .messages
            .last_mut()
            .ok_or("There is no interrupted reply")?;
        if reply.status != MessageStatus::Interrupted {
            return Err("Only interrupted work can Resume");
        }
        if reply
            .steps
            .iter()
            .flat_map(|step| &step.calls)
            .any(|call| call.status == ToolStatus::Unknown)
        {
            return Err(
                "An uncertain tool outcome requires inspection; submit a new instruction after acknowledging the interruption",
            );
        }
        reply.status = MessageStatus::InProgress;
        if let Some(run) = self.runs.last_mut() {
            if run.reply_id != reply.id || run.status != RunStatus::Interrupted {
                return Err("Interrupted run does not match its conversation");
            }
            run.status = RunStatus::Running;
        }
        self.resume_required = false;
        Ok(())
    }

    pub fn acknowledge_interruption(&mut self) -> Result<(), &'static str> {
        if !self.resume_required {
            return Err("There is no interruption to acknowledge");
        }
        self.resume_required = false;
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
        if let Some(run) = self.runs.last_mut() {
            run.status = match status {
                MessageStatus::Cancelled => RunStatus::Stopped,
                MessageStatus::Failed => RunStatus::Failed,
                MessageStatus::Interrupted => RunStatus::Interrupted,
                _ => RunStatus::NeedsHuman,
            };
        }
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
        if changed {
            self.resume_required = true;
            if let Some(run) = self.runs.last_mut()
                && run.status == RunStatus::Running
            {
                run.status = RunStatus::Interrupted;
            }
        }
        changed
    }
}
