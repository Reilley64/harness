use crate::CapturedJoin;
use crate::ChildExecution;
use crate::ChildExecutionStatus;
use crate::ChildInstruction;
use crate::ChildReceipt;
use crate::CoordinationCommand;
use crate::CoordinationGate;
use crate::ScopeAmendment;

#[derive(Debug, Clone, PartialEq)]
pub struct Coordination {
    pub conversation_id: String,
    pub agent_id: String,
    pub revision: u64,
    pub authority_complete: bool,

    pub gate: CoordinationGate,
    pub controller: Option<String>,
    pub run_id: Option<String>,
    pub waiting: bool,

    pub children: Vec<ChildExecution>,
    pub joins: Vec<CapturedJoin>,
    pub instructions: Vec<ChildInstruction>,
    pub recovered: Vec<String>,
    pub scopes: Vec<ScopeAmendment>,
    pub restart_required: bool,
    pub recovery_notices: Vec<String>,
}

impl Coordination {
    pub fn new(conversation_id: String, agent_id: String) -> Self {
        Self {
            conversation_id,
            agent_id,
            revision: 0,
            authority_complete: true,

            gate: CoordinationGate::Open,
            controller: None,
            run_id: None,
            waiting: false,

            children: Vec::new(),
            joins: Vec::new(),
            instructions: Vec::new(),
            recovered: Vec::new(),
            scopes: Vec::new(),
            restart_required: false,
            recovery_notices: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.conversation_id.is_empty()
            || self.conversation_id.len() > 128
            || self.agent_id.is_empty()
            || self.agent_id.len() > 128
            || self.controller.is_some() != self.run_id.is_some()
            || self.waiting && self.controller.is_none()
            || self
                .controller
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self
                .run_id
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self.children.len() > 256
            || self.joins.len() > 256
            || self.instructions.len() > 256
            || self.recovered.len() > 256
            || self.scopes.len() > 256
            || self.recovery_notices.len() > 4096
            || self.restart_required && self.recovery_notices.is_empty()
        {
            return Err(
                "Coordination facts are missing or exceed bounded storage; inspect retained work",
            );
        }
        for (index, child) in self.children.iter().enumerate() {
            child.validate()?;
            if child.resume_required && !self.recovered.contains(&child.execution_id) {
                return Err("Child recovery gate lacks its retained recovery identity");
            }
            if child.parent_id != self.conversation_id
                || child.parent_agent_id != self.agent_id
                || self.children[..index].iter().any(|prior| {
                    prior.intent == child.intent
                        || prior.assignment_id == child.assignment_id
                        || prior.agent_id == child.agent_id
                        || prior.execution_id == child.execution_id
                        || prior
                            .outcome
                            .as_ref()
                            .zip(child.outcome.as_ref())
                            .is_some_and(|(a, b)| a.id == b.id)
                })
            {
                return Err("Child identity is duplicated or belongs to another parent");
            }
        }
        for (index, notice) in self.recovery_notices.iter().enumerate() {
            if notice.is_empty()
                || notice.len() > 128
                || self.recovery_notices[..index].contains(notice)
            {
                return Err("Recovery notice history is invalid or duplicated");
            }
        }
        for (index, scope) in self.scopes.iter().enumerate() {
            scope.validate()?;
            if !self
                .children
                .iter()
                .any(|child| child.execution_id == scope.execution_id)
                || self.scopes[..index]
                    .iter()
                    .any(|prior| prior.id == scope.id || prior.version == scope.version)
                || scope.replaces.as_deref()
                    != self.scopes[..index]
                        .iter()
                        .rev()
                        .find(|prior| prior.execution_id == scope.execution_id)
                        .map(|prior| prior.id.as_str())
            {
                return Err("Scope amendment history has conflicting ownership or supersession");
            }
        }
        for (index, execution) in self.recovered.iter().enumerate() {
            if self.recovered[..index].contains(execution)
                || !self
                    .children
                    .iter()
                    .any(|child| &child.execution_id == execution)
            {
                return Err("Recovered ownership is duplicated or missing");
            }
        }
        for (index, child) in self.children.iter().enumerate() {
            if child.receipts().any(|(outcome, _, _)| {
                self.children[..index].iter().any(|prior| {
                    prior
                        .receipts()
                        .any(|(receipt, _, _)| receipt.id == outcome.id)
                })
            }) {
                return Err("Receipt identity is duplicated across execution owners");
            }
        }
        for (index, instruction) in self.instructions.iter().enumerate() {
            instruction.validate()?;
            if instruction.order != index as u64 + 1
                || !self
                    .children
                    .iter()
                    .any(|child| child.execution_id == instruction.execution_id)
                || self.instructions[..index]
                    .iter()
                    .any(|prior| prior.id == instruction.id)
            {
                return Err("Child instruction order or exact ownership is invalid");
            }
        }
        for (index, join) in self.joins.iter().enumerate() {
            if join.id.is_empty()
                || join.id.len() > 128
                || join.run_id.is_empty()
                || join.run_id.len() > 128
                || join.executions.is_empty()
                || join.executions.len() > 256
                || self.joins[..index].iter().any(|prior| prior.id == join.id)
            {
                return Err("Captured join identity or membership is invalid");
            }
            for (index, id) in join.executions.iter().enumerate() {
                if join.executions[..index].contains(id)
                    || !self.children.iter().any(|child| &child.execution_id == id)
                {
                    return Err("Captured join has duplicate or missing required executions");
                }
            }
        }
        Ok(())
    }

    pub fn scope(&self, execution_id: &str) -> Option<&ScopeAmendment> {
        self.scopes
            .iter()
            .rev()
            .find(|scope| scope.execution_id == execution_id)
    }

    pub fn execution_gate(&self, execution_id: &str) -> Result<CoordinationGate, &'static str> {
        if !self.authority_complete {
            return Err("Execution ownership authority is incomplete");
        }
        let child = self
            .children
            .iter()
            .find(|child| child.execution_id == execution_id)
            .ok_or("Execution ownership is missing")?;
        Ok(
            if child.stop_requested || self.gate == CoordinationGate::Stopped {
                CoordinationGate::Stopped
            } else if child.resume_required || self.gate == CoordinationGate::ResumeRequired {
                CoordinationGate::ResumeRequired
            } else {
                self.gate
            },
        )
    }

    pub fn settled(&self) -> bool {
        self.authority_complete
            && !self.restart_required
            && self.controller.is_none()
            && self
                .scopes
                .iter()
                .filter(|scope| {
                    self.scope(&scope.execution_id)
                        .is_some_and(|latest| latest.id == scope.id)
                })
                .all(|scope| scope.approved_by.is_some())
            && self.children.iter().all(|child| {
                child.status != ChildExecutionStatus::NeedsHuman
                    && !child.resume_required
                    && child.outcome.is_some()
                    && child.follow_up_completed_by.is_some()
                    && child
                        .history
                        .iter()
                        .all(|receipt| receipt.follow_up_completed_by.is_some())
            })
    }

    pub fn eligible(&self, run_id: &str) -> Result<bool, &'static str> {
        if !self.authority_complete || self.restart_required || self.gate != CoordinationGate::Open
        {
            return Ok(false);
        }
        for join in self.joins.iter().filter(|join| join.run_id == run_id) {
            if !join.ready(&self.children)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn request_scope(&mut self, scope: ScopeAmendment) -> Result<bool, &'static str> {
        scope.validate()?;
        if let Some(prior) = self.scopes.iter().find(|prior| prior.id == scope.id) {
            return if prior.execution_id == scope.execution_id
                && prior.text == scope.text
                && prior.replaces == scope.replaces
                && prior.supersedes_run == scope.supersedes_run
            {
                Ok(false)
            } else {
                Err("Scope request identity is already bound to different exact criteria")
            };
        }
        let child = self
            .children
            .iter()
            .find(|child| child.execution_id == scope.execution_id)
            .ok_or("Scope amendment has no execution owner")?;
        if !self.authority_complete
            || !matches!(
                self.gate,
                CoordinationGate::Open | CoordinationGate::NeedsHuman
            )
            || child.stop_requested
            || child.resume_required
            || scope.approved_by.is_some()
            || !matches!(
                child.status,
                ChildExecutionStatus::Running | ChildExecutionStatus::NeedsHuman
            )
            || self
                .instructions
                .iter()
                .any(|instruction| instruction.id == scope.id)
        {
            return Err(
                "Affected execution is stopped, recovered, terminal or lacks scope authority",
            );
        }
        if child.status == ChildExecutionStatus::NeedsHuman {
            if scope.supersedes_run.is_none()
                || scope.supersedes_run.as_deref()
                    != child
                        .outcome
                        .as_ref()
                        .and_then(|outcome| outcome.run_id.as_deref())
            {
                return Err(
                    "Obsolete input must be explicitly superseded by its exact requesting run",
                );
            }
        } else if scope.supersedes_run.is_some() {
            return Err("Active scope amendment cannot silently supersede another input request");
        }
        self.scopes.push(scope);
        Ok(true)
    }

    fn approve_scope(
        &mut self,
        id: String,
        version: String,
        approval_id: String,
    ) -> Result<bool, &'static str> {
        let index = self
            .scopes
            .iter()
            .position(|scope| scope.id == id)
            .ok_or("Scope request is missing")?;
        let scope = &self.scopes[index];
        if self
            .scope(&scope.execution_id)
            .is_none_or(|current| current.id != id)
            || scope.version != version
        {
            return Err("Only the latest exact scope version can be approved");
        }
        if let Some(prior) = &scope.approved_by {
            return if prior == &approval_id {
                Ok(false)
            } else {
                Err("Scope version is already bound to another approval identity")
            };
        }
        let child = self
            .children
            .iter()
            .find(|child| child.execution_id == scope.execution_id)
            .ok_or("Scope approval lost its execution owner")?;
        if child.stop_requested
            || child.resume_required
            || !self.authority_complete
            || !matches!(
                self.gate,
                CoordinationGate::Open | CoordinationGate::NeedsHuman
            )
        {
            return Err("Scope approval cannot clear Stop, recovery or incomplete ownership");
        }
        if child.status == ChildExecutionStatus::Running {
            if self
                .instructions
                .iter()
                .filter(|instruction| instruction.execution_id == child.execution_id)
                .count()
                >= 64
            {
                return Err(
                    "Instruction capacity is full; scope approval cannot drop prior accepted instructions",
                );
            }
            self.instructions.push(ChildInstruction {
                id: scope.id.clone(),
                execution_id: scope.execution_id.clone(),
                order: self.instructions.len() as u64 + 1,
                text: format!(
                    "Trusted human approval {} for exact assignment scope version {}. Criteria: {}",
                    approval_id, scope.version, scope.text
                ),
                applied_run: None,
                applied_reply: None,
            });
        }
        self.scopes[index].approved_by = Some(approval_id);
        Ok(true)
    }

    fn admit_child_action(
        &self,
        execution_id: &str,
        controller: &str,
        descendant: bool,
    ) -> Result<bool, &'static str> {
        let child = self
            .children
            .iter()
            .find(|child| child.execution_id == execution_id)
            .ok_or("Child action has no durable execution owner")?;
        let status = child.status == ChildExecutionStatus::Running
            || descendant
                && matches!(
                    child.status,
                    ChildExecutionStatus::NeedsHuman | ChildExecutionStatus::Completed
                );
        if !self.authority_complete
            || !matches!(
                self.gate,
                CoordinationGate::Open | CoordinationGate::NeedsHuman
            )
            || child.controller != controller
            || child.stop_requested
            || child.resume_required
            || !status
            || self
                .scope(execution_id)
                .is_some_and(|scope| scope.approved_by.is_none())
            || self.instructions.iter().any(|instruction| {
                instruction.execution_id == execution_id && instruction.applied_run.is_none()
            })
        {
            return Err(
                "Child action is fenced by ownership, Stop, recovery, scope approval or unapplied human instructions",
            );
        }
        Ok(true)
    }

    fn controls(&self, controller: &str, run_id: &str) -> Result<(), &'static str> {
        if self.controller.as_deref() != Some(controller) || self.run_id.as_deref() != Some(run_id)
        {
            return Err("Another execution controls this durable agent; inspect without replay");
        }
        Ok(())
    }

    pub fn apply(&mut self, command: CoordinationCommand) -> Result<bool, &'static str> {
        let mut next = self.clone();
        let changed = next.transition(command)?;
        next.validate()?;
        if changed {
            *self = next;
        }
        Ok(changed)
    }

    fn transition(&mut self, command: CoordinationCommand) -> Result<bool, &'static str> {
        match command {
            CoordinationCommand::RecoverChild {
                execution_id,
                notice_id,
            } => {
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Lost execution owner is missing")?;
                if child.outcome.is_some() {
                    return Ok(false);
                }
                if !matches!(
                    child.status,
                    ChildExecutionStatus::Held | ChildExecutionStatus::Running
                ) && !child.resume_required
                {
                    return Ok(false);
                }
                if child.resume_required && child.recovery_notice.is_some() {
                    return Ok(false);
                }
                child.status = ChildExecutionStatus::Unknown;
                child.resume_required = !child.stop_requested;
                child.recovery_notice = Some(notice_id);
                if !self.recovered.contains(&execution_id) {
                    self.recovered.push(execution_id);
                }
            }
            CoordinationCommand::RetainRecovery {
                agent_id,
                notices,
                idle,
            } => {
                if agent_id != self.agent_id || !self.authority_complete || notices.is_empty() {
                    return Err("Recovery does not match complete durable ownership");
                }
                let mut changed = false;
                for notice in notices {
                    if !self.recovery_notices.contains(&notice) {
                        self.recovery_notices.push(notice);
                        changed = true;
                    }
                }
                if changed && idle {
                    self.restart_required = true;
                    self.controller = None;
                    self.run_id = None;
                    self.waiting = false;
                    if self.gate == CoordinationGate::Open {
                        self.gate = CoordinationGate::NeedsHuman;
                    }
                }
                return Ok(changed);
            }
            CoordinationCommand::ResumeIdle { agent_id } => {
                if agent_id != self.agent_id || !self.restart_required || self.controller.is_some()
                {
                    return Err("Idle recovery requires its exact explicit Resume acknowledgement");
                }
                self.restart_required = false;
            }
            CoordinationCommand::RequestScope(scope) => return self.request_scope(scope),
            CoordinationCommand::ApproveScope {
                id,
                version,
                approval_id,
            } => return self.approve_scope(id, version, approval_id),
            CoordinationCommand::SteerChild {
                id,
                execution_id,
                text,
            } => {
                if self
                    .scope(&execution_id)
                    .is_some_and(|scope| scope.approved_by.is_none())
                    || self.scopes.iter().any(|scope| scope.id == id)
                {
                    return Err(
                        "Only the matching exact-version approval can satisfy this scope gate",
                    );
                }
                if let Some(prior) = self.instructions.iter().find(|prior| prior.id == id) {
                    return if prior.execution_id == execution_id && prior.text == text {
                        Ok(false)
                    } else {
                        Err("Instruction identity is already bound to different scope or text")
                    };
                }
                let child = self
                    .children
                    .iter()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Targeted instruction does not name an owned execution")?;
                if !self.authority_complete
                    || !matches!(
                        self.gate,
                        CoordinationGate::Open | CoordinationGate::NeedsHuman
                    )
                    || child.stop_requested
                    || child.resume_required
                    || child.status != ChildExecutionStatus::Running
                    || self
                        .instructions
                        .iter()
                        .filter(|prior| prior.execution_id == execution_id)
                        .count()
                        >= 64
                {
                    return Err(
                        "Targeted child is stopped, input/approval/restart gated or no longer active",
                    );
                }
                self.instructions.push(ChildInstruction {
                    id,
                    execution_id,
                    order: self.instructions.len() as u64 + 1,
                    text,
                    applied_run: None,
                    applied_reply: None,
                });
            }
            CoordinationCommand::ResumeChild { execution_id } => {
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Recovered child execution is missing")?;
                if !self.authority_complete
                    || !matches!(
                        self.gate,
                        CoordinationGate::Open | CoordinationGate::NeedsHuman
                    )
                    || child.stop_requested
                    || !child.resume_required
                    || child.status != ChildExecutionStatus::Unknown
                    || child.outcome.is_some()
                {
                    return Err(
                        "Only a recovered, explicitly resumed child may continue; Stop and uncertain results remain fenced",
                    );
                }
                child.resume_required = false;
                child.status = ChildExecutionStatus::Running;
            }
            CoordinationCommand::AnswerChild {
                execution_id,
                request_run,
                approval_id,
            } => {
                if let Some(scope) = self.scope(&execution_id) {
                    if scope.approved_by.is_none()
                        || (scope.supersedes_run.as_deref() == Some(request_run.as_str())
                            && scope.approved_by != approval_id)
                    {
                        return Err(
                            "Only exact-version approval can supersede this obsolete input; updates and answers cannot clear scope approval",
                        );
                    }
                } else if approval_id.is_some() {
                    return Err(
                        "Answer supplied an approval identity without retained scope authority",
                    );
                }
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Human answer does not name an owned execution")?;
                let outcome = child
                    .outcome
                    .as_ref()
                    .ok_or("Human input request has no durable receipt")?;
                if !self.authority_complete
                    || !matches!(
                        self.gate,
                        CoordinationGate::Open | CoordinationGate::NeedsHuman
                    )
                    || child.stop_requested
                    || child.resume_required
                    || child.status != ChildExecutionStatus::NeedsHuman
                    || outcome.run_id.as_deref() != Some(request_run.as_str())
                    || child.history.len() >= 64
                {
                    return Err(
                        "Answer does not match the pending human-input run, or execution is gated",
                    );
                }
                child.history.push(ChildReceipt {
                    outcome: child.outcome.take().ok_or("Input receipt disappeared")?,
                    delivered_to: child.delivered_to.take(),
                    follow_up_completed_by: child.follow_up_completed_by.take(),
                });
                child.status = ChildExecutionStatus::Running;
            }
            CoordinationCommand::AdmitChildAction {
                execution_id,
                controller,
            } => return self.admit_child_action(&execution_id, &controller, false),
            CoordinationCommand::AdmitDescendantAction {
                execution_id,
                controller,
            } => return self.admit_child_action(&execution_id, &controller, true),
            CoordinationCommand::ApplyChildInstruction {
                id,
                controller,
                run_id,
                reply_id,
            } => {
                let instruction = self
                    .instructions
                    .iter_mut()
                    .find(|instruction| instruction.id == id)
                    .ok_or("Accepted child instruction is missing")?;
                let child = self
                    .children
                    .iter()
                    .find(|child| child.execution_id == instruction.execution_id)
                    .ok_or("Accepted child instruction has no execution owner")?;
                if child.controller != controller {
                    return Err(
                        "Instruction acknowledgement belongs to a different execution controller",
                    );
                }
                if instruction.applied_run.is_some() {
                    return if instruction.applied_run.as_deref() == Some(&run_id)
                        && instruction.applied_reply.as_deref() == Some(&reply_id)
                    {
                        Ok(false)
                    } else {
                        Err("Instruction was already checkpointed by a different run or reply")
                    };
                }
                instruction.applied_run = Some(run_id);
                instruction.applied_reply = Some(reply_id);
            }
            CoordinationCommand::Dispatch(child) => {
                if let Some(prior) = self
                    .children
                    .iter()
                    .find(|prior| prior.intent == child.intent)
                {
                    return if prior.same_assignment(&child) {
                        Ok(false)
                    } else {
                        Err("Dispatch key is bound to a different exact typed payload")
                    };
                }
                if self.gate != CoordinationGate::Open
                    || self.controller.is_none()
                    || self.run_id.as_deref() != Some(child.parent_run_id.as_str())
                    || child.status != ChildExecutionStatus::Held
                    || child.stop_requested
                    || child.outcome.is_some()
                    || child.delivered_to.is_some()
                    || child.follow_up_completed_by.is_some()
                {
                    return Err("Dispatch is gated or initial execution facts are invalid");
                }
                self.children.push(*child);
            }
            CoordinationCommand::RetainClaim(child) => {
                if let Some(prior) = self
                    .children
                    .iter()
                    .find(|prior| prior.intent == child.intent)
                {
                    return if prior.same_assignment(&child) {
                        Ok(false)
                    } else {
                        Err("Retained claim conflicts with its accepted payload")
                    };
                }
                if child.status != ChildExecutionStatus::Unknown
                    || child.outcome.is_none()
                    || child.delivered_to.is_some()
                    || child.follow_up_completed_by.is_some()
                {
                    return Err(
                        "Held-claim reconciliation must retain an undelivered Unknown receipt without replay",
                    );
                }
                self.children.push(*child);
            }
            CoordinationCommand::Start {
                execution_id,
                controller,
            } => {
                if !matches!(
                    self.gate,
                    CoordinationGate::Open | CoordinationGate::NeedsHuman
                ) {
                    return Err("Child start is gated; retain the claim without replay");
                }
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Child execution is missing")?;
                if child.stop_requested
                    || child.controller != controller
                    || child.status != ChildExecutionStatus::Held
                {
                    return Err(
                        "Child already started or belongs to another controller; never replay",
                    );
                }
                child.status = ChildExecutionStatus::Running;
            }
            CoordinationCommand::MarkUnknown {
                execution_id,
                controller,
            } => {
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Unknown execution")?;
                if child.controller != controller {
                    return Err("Execution controller differs");
                }
                if child.outcome.is_some() {
                    return Ok(false);
                }
                if child.status == ChildExecutionStatus::Unknown && child.stop_requested {
                    return Ok(false);
                }
                child.status = ChildExecutionStatus::Unknown;
                child.resume_required = false;
                child.stop_requested = true;
            }
            CoordinationCommand::StopChild { execution_id } => {
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Child execution is missing")?;
                if child.stop_requested {
                    return Ok(false);
                }
                child.stop_requested = true;
            }
            CoordinationCommand::Stop { agent_id } => {
                if agent_id != self.agent_id {
                    return Err("Stop belongs to a different durable agent");
                }
                let changed = self.gate != CoordinationGate::Stopped
                    || self.children.iter().any(|child| !child.stop_requested);
                self.gate = CoordinationGate::Stopped;
                for child in &mut self.children {
                    child.stop_requested = true;
                }
                return Ok(changed);
            }
            CoordinationCommand::Receive {
                execution_id,
                controller,
                outcome,
            } => {
                outcome.validate()?;
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Child execution is missing")?;
                if child.controller != controller {
                    return Err("Outcome sender does not control the accepted execution");
                }
                if let Some(prior) = &child.outcome {
                    return if prior == &outcome {
                        Ok(false)
                    } else {
                        Err("Outcome identity is already bound to different facts")
                    };
                }
                if !matches!(
                    child.status,
                    ChildExecutionStatus::Held
                        | ChildExecutionStatus::Running
                        | ChildExecutionStatus::Unknown
                ) {
                    return Err("Execution cannot receive another terminal outcome");
                }
                child.status = outcome.status;
                child.resume_required = false;
                child.outcome = Some(outcome);
            }
            CoordinationCommand::Join(join) => {
                if let Some(prior) = self.joins.iter().find(|prior| prior.id == join.id) {
                    return if prior.same_intent(&join) {
                        Ok(false)
                    } else {
                        Err("Join key is already bound to a fixed execution set")
                    };
                }
                if self.gate != CoordinationGate::Open || join.backgrounded {
                    return Err("Join is gated or already backgrounded");
                }
                self.joins.push(join);
            }
            CoordinationCommand::Background { join_id, run_id } => {
                let join = self
                    .joins
                    .iter_mut()
                    .find(|join| join.id == join_id)
                    .ok_or("Captured join is missing")?;
                if join.run_id != run_id {
                    return Err("Background request belongs to another logical run");
                }
                if join.backgrounded {
                    return Ok(false);
                }
                join.backgrounded = true;
            }
            CoordinationCommand::Acquire {
                agent_id,
                controller,
                run_id,
            } => {
                if agent_id != self.agent_id {
                    return Err("Controlling execution belongs to a different durable agent");
                }
                if self.gate != CoordinationGate::Open {
                    return Err("Parent execution is gated; inspect without inference");
                }
                if self.controller.is_some() {
                    self.controls(&controller, &run_id)?;
                    return Ok(false);
                }
                if !self.eligible(&run_id)? {
                    return Err("Parent continuation is gated or captured join remains pending");
                }
                self.controller = Some(controller);
                self.run_id = Some(run_id);
            }
            CoordinationCommand::AcknowledgeParent {
                agent_id,
                controller,
                run_id,
            } => {
                self.controls(&controller, &run_id)?;
                if agent_id != self.agent_id
                    || !matches!(
                        self.gate,
                        CoordinationGate::ResumeRequired | CoordinationGate::Stopped
                    )
                    || !self.authority_complete
                {
                    return Err(
                        "Only the recovered controlling run can acknowledge this interruption",
                    );
                }
                for child in &mut self.children {
                    if child.outcome.is_none() {
                        child.stop_requested = true;
                    }
                }
                if self.gate != CoordinationGate::Stopped {
                    self.gate = CoordinationGate::NeedsHuman;
                }
                self.controller = None;
                self.run_id = None;
                self.waiting = false;
            }
            CoordinationCommand::ResumeInputParent {
                agent_id,
                controller,
                run_id,
            } => {
                self.controls(&controller, &run_id)?;
                if agent_id != self.agent_id
                    || self.gate != CoordinationGate::ResumeRequired
                    || !self.authority_complete
                {
                    return Err(
                        "Recovered human input requires its exact Resume gate; Stop cannot be cleared",
                    );
                }
                self.gate = CoordinationGate::NeedsHuman;
                self.controller = None;
                self.run_id = None;
                self.waiting = false;
            }
            CoordinationCommand::ResumeParent {
                agent_id,
                controller,
                run_id,
            } => {
                self.controls(&controller, &run_id)?;
                if agent_id != self.agent_id
                    || self.gate != CoordinationGate::ResumeRequired
                    || !self.authority_complete
                {
                    return Err(
                        "Only the recovered controlling run can explicitly Resume this parent",
                    );
                }
                self.gate = CoordinationGate::Open;
                self.controller = None;
                self.run_id = None;
                self.waiting = false;
            }
            CoordinationCommand::Wait { controller, run_id } => {
                self.controls(&controller, &run_id)?;
                if self.gate != CoordinationGate::Open {
                    return Err("A closed gate cannot begin a delegated wait");
                }
                if self.waiting {
                    return Ok(false);
                }
                self.waiting = true;
            }
            CoordinationCommand::Deliver {
                controller,
                run_id,
                outcomes,
            } => {
                self.controls(&controller, &run_id)?;
                if !self.eligible(&run_id)? {
                    return Err("Parent delivery is gated or captured join remains pending");
                }
                let mut changed = self.waiting;
                self.waiting = false;
                for id in outcomes {
                    let child = self
                        .children
                        .iter_mut()
                        .find(|child| child.receipts().any(|(outcome, _, _)| outcome.id == id))
                        .ok_or("Required outcome receipt is unavailable; inspect coordination")?;
                    changed |= child.deliver(&id, &run_id)?;
                }
                return Ok(changed);
            }
            CoordinationCommand::CompleteFollowUp {
                controller,
                run_id,
                outcomes,
            } => {
                self.controls(&controller, &run_id)?;
                if !self.eligible(&run_id)? {
                    return Err("Follow-up cannot finish through a closed gate or pending join");
                }
                let mut changed = false;
                for id in outcomes {
                    let child = self
                        .children
                        .iter_mut()
                        .find(|child| child.receipts().any(|(outcome, _, _)| outcome.id == id))
                        .ok_or("Required outcome receipt is unavailable")?;
                    changed |= child.complete_follow_up(&id, &run_id)?;
                }
                return Ok(changed);
            }
            CoordinationCommand::Release { controller, run_id } => {
                self.controls(&controller, &run_id)?;
                self.controller = None;
                self.run_id = None;
                self.waiting = false;
            }
            CoordinationCommand::Gate {
                gate,
                controller,
                run_id,
            } => {
                if gate == CoordinationGate::Open {
                    if !self.authority_complete
                        || self.restart_required
                        || controller.is_some()
                        || run_id.is_some()
                        || self.controller.is_some()
                        || !matches!(
                            self.gate,
                            CoordinationGate::Open
                                | CoordinationGate::NeedsHuman
                                | CoordinationGate::Stopped
                                | CoordinationGate::Unknown
                        )
                    {
                        return Err(
                            "Opening a parent gate requires a fresh eligible prompt admission",
                        );
                    }
                } else {
                    self.controls(
                        controller
                            .as_deref()
                            .ok_or("Closing gate controller is missing")?,
                        run_id.as_deref().ok_or("Closing gate run is missing")?,
                    )?;
                }
                if self.gate == gate
                    || self.gate == CoordinationGate::Stopped && gate != CoordinationGate::Open
                {
                    return Ok(false);
                }
                if gate == CoordinationGate::ResumeRequired {
                    if self.recovery_notices.is_empty() {
                        return Err(
                            "Recovery requires a retained intent before closing execution ownership",
                        );
                    }
                    for child in &mut self.children {
                        if child.outcome.is_none() {
                            child.status = ChildExecutionStatus::Unknown;
                            if !child.resume_required {
                                child.recovery_notice = self.recovery_notices.last().cloned();
                            }
                            child.resume_required = true;
                            if !self.recovered.contains(&child.execution_id) {
                                self.recovered.push(child.execution_id.clone());
                            }
                        }
                    }
                }
                if gate == CoordinationGate::Stopped {
                    for child in &mut self.children {
                        if child.outcome.is_none() {
                            child.stop_requested = true;
                        }
                    }
                }
                self.gate = gate;
            }
        }
        Ok(true)
    }
}
