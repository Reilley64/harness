use crate::CapturedJoin;
use crate::ChildExecution;
use crate::ChildExecutionStatus;
use crate::CoordinationCommand;
use crate::CoordinationGate;

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
        {
            return Err(
                "Coordination facts are missing or exceed bounded storage; inspect retained work",
            );
        }
        for (index, child) in self.children.iter().enumerate() {
            child.validate()?;
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

    pub fn settled(&self) -> bool {
        self.authority_complete
            && self.controller.is_none()
            && self
                .children
                .iter()
                .all(|child| child.outcome.is_some() && child.follow_up_completed_by.is_some())
    }

    pub fn eligible(&self, run_id: &str) -> Result<bool, &'static str> {
        if !self.authority_complete || self.gate != CoordinationGate::Open {
            return Ok(false);
        }
        for join in self.joins.iter().filter(|join| join.run_id == run_id) {
            if !join.ready(&self.children)? {
                return Ok(false);
            }
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
                        .find(|child| {
                            child
                                .outcome
                                .as_ref()
                                .is_some_and(|outcome| outcome.id == id)
                        })
                        .ok_or("Required outcome receipt is unavailable; inspect coordination")?;
                    if child.delivered_to.is_none() {
                        child.delivered_to = Some(run_id.clone());
                        changed = true;
                    }
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
                        .find(|child| {
                            child
                                .outcome
                                .as_ref()
                                .is_some_and(|outcome| outcome.id == id)
                        })
                        .ok_or("Required outcome receipt is unavailable")?;
                    if child.delivered_to.is_none() {
                        return Err("Receipt alone is not delivery or completed follow-up");
                    }
                    if child.follow_up_completed_by.is_none() {
                        child.follow_up_completed_by = Some(run_id.clone());
                        changed = true;
                    }
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
                    for child in &mut self.children {
                        if child.outcome.is_none() {
                            child.status = ChildExecutionStatus::Unknown;
                            child.stop_requested = true;
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
