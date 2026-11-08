use crate::CapturedJoin;
use crate::ChildExecution;
use crate::ChildExecutionStatus;
use crate::CoordinationCommand;
use crate::CoordinationGate;

#[derive(Debug, Clone, PartialEq)]
pub struct Coordination {
    pub agent_id: String,
    pub revision: u64,

    pub gate: CoordinationGate,
    pub controller: Option<String>,
    pub run_id: Option<String>,

    pub children: Vec<ChildExecution>,
    pub joins: Vec<CapturedJoin>,
}

impl Coordination {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            revision: 0,
            gate: CoordinationGate::Open,
            controller: None,
            run_id: None,
            children: Vec::new(),
            joins: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.agent_id.is_empty()
            || self.agent_id.len() > 128
            || self.controller.is_some() != self.run_id.is_some()
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
            if child.parent_id != self.agent_id
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
        self.controller.is_none()
            && self
                .children
                .iter()
                .all(|child| child.outcome.is_some() && child.follow_up_completed_by.is_some())
    }

    pub fn eligible(&self, run_id: &str) -> Result<bool, &'static str> {
        if self.gate != CoordinationGate::Open {
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
                    || child.status != ChildExecutionStatus::Held
                    || child.outcome.is_some()
                    || child.delivered_to.is_some()
                    || child.follow_up_completed_by.is_some()
                {
                    return Err("Dispatch is gated or initial execution facts are invalid");
                }
                self.children.push(*child);
            }
            CoordinationCommand::Start {
                execution_id,
                controller,
            } => {
                if self.gate != CoordinationGate::Open {
                    return Err("Child start is gated; retain the claim without replay");
                }
                let child = self
                    .children
                    .iter_mut()
                    .find(|child| child.execution_id == execution_id)
                    .ok_or("Child execution is missing")?;
                if child.controller != controller || child.status != ChildExecutionStatus::Held {
                    return Err(
                        "Child already started or belongs to another controller; never replay",
                    );
                }
                child.status = ChildExecutionStatus::Running;
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
                    ChildExecutionStatus::Held | ChildExecutionStatus::Running
                ) {
                    return Err("Execution cannot receive another terminal outcome");
                }
                child.status = outcome.status;
                child.outcome = Some(outcome);
            }
            CoordinationCommand::Join(join) => {
                if let Some(prior) = self.joins.iter().find(|prior| prior.id == join.id) {
                    return if prior == &join {
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
            CoordinationCommand::Acquire { controller, run_id } => {
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
            CoordinationCommand::Deliver {
                controller,
                run_id,
                outcomes,
            } => {
                self.controls(&controller, &run_id)?;
                if !self.eligible(&run_id)? {
                    return Err("Parent delivery is gated or captured join remains pending");
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
            }
            CoordinationCommand::Gate { gate } => {
                if self.gate == gate {
                    return Ok(false);
                }
                self.gate = gate;
            }
        }
        Ok(true)
    }
}
