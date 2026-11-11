use crate::AssignmentBrief;
use crate::DecisionCandidate;
use crate::DecisionPhase;
use crate::DecisionRequest;
use crate::DecisionResponse;
use crate::DecisionStage;
use crate::ModelSelection;

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentDecision {
    pub id: String,
    pub parent_id: String,
    pub parent_agent_id: String,
    pub parent_revision: u64,
    pub parent_run_id: Option<String>,
    pub parent: ModelSelection,
    pub parent_connection_id: String,
    pub work_id: String,
    pub work_intent: String,
    pub work_version: String,
    pub brief: AssignmentBrief,

    pub legacy: bool,
    pub revision: u64,
    pub finished_at: Option<u64>,
    pub phase: DecisionPhase,
    pub request: Option<DecisionRequest>,
    pub response: Option<DecisionResponse>,
    pub stages: Vec<DecisionStage>,
    pub selected: Option<DecisionCandidate>,
    pub reason: Option<String>,
}

impl AssignmentDecision {
    pub fn same_intent(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent_id == other.parent_id
            && self.parent_agent_id == other.parent_agent_id
            && self.work_id == other.work_id
            && self.work_intent == other.work_intent
            && self.work_version == other.work_version
            && self.brief == other.brief
    }

    pub fn stop(&mut self) -> Result<(), &'static str> {
        if !matches!(self.phase, DecisionPhase::Prepared | DecisionPhase::Pending) {
            return Err("Only unsettled native work can be stopped");
        }
        self.response = self
            .response
            .take()
            .map(DecisionResponse::into_report)
            .transpose()?;
        for stage in self.stages.iter_mut().filter(|s| s.finished_at.is_none()) {
            stage.response = stage
                .response
                .take()
                .map(DecisionResponse::into_report)
                .transpose()?;
            stage.failure = Some(
                if stage.started_at.is_some() {
                    "Explicit Stop; native outcome may be unknown"
                } else {
                    "Explicit Stop before stage admission; no native attempt"
                }
                .into(),
            );
        }
        self.selected = None;
        self.phase = DecisionPhase::Stopped;
        Ok(())
    }

    pub fn retain_stop(&mut self, stopped: Self) -> Result<(), &'static str> {
        self.validate()?;
        stopped.validate()?;
        if stopped.phase != DecisionPhase::Stopped
            || !self.same_intent(&stopped)
            || self.parent != stopped.parent
            || self.parent_connection_id != stopped.parent_connection_id
            || self.parent_revision != stopped.parent_revision
            || self.parent_run_id != stopped.parent_run_id
            || self.legacy != stopped.legacy
            || stopped.revision > self.revision
            || stopped
                .request
                .as_ref()
                .is_some_and(|request| Some(request) != self.request.as_ref())
        {
            return Err("Explicit Stop differs from the unsettled intent");
        }
        let response = match (&self.response, stopped.response) {
            (Some(existing), Some(report)) if existing != &report => {
                return Err("Explicit Stop carries conflicting native accounting");
            }
            (Some(existing), _) => Some(existing.clone()),
            (None, report) => report,
        };

        self.stop()?;
        self.response = response;
        self.finished_at = stopped.finished_at;
        self.reason = stopped.reason;
        self.validate()
    }

    pub fn retain_stopped_report(&mut self, completed: Self) -> Result<bool, &'static str> {
        if self.phase != DecisionPhase::Stopped
            || !self.same_intent(&completed)
            || self.parent != completed.parent
            || self.parent_connection_id != completed.parent_connection_id
            || self.parent_revision != completed.parent_revision
            || self.parent_run_id != completed.parent_run_id
            || self.legacy != completed.legacy
            || self.request != completed.request
        {
            return Err("Late native report differs from the stopped request");
        }
        if self.stages.len() != completed.stages.len()
            || self
                .stages
                .iter()
                .zip(&completed.stages)
                .any(|(a, b)| !a.same_input(b))
        {
            return Err("Late native stage report differs from the stopped inputs");
        }
        let Some(report) = completed.response else {
            return Ok(false);
        };
        if self.request.is_none() {
            return Err("Stopped native report is missing its request");
        }
        let report = report.into_report()?;
        if !self.stages.is_empty() {
            let source = completed
                .stages
                .iter()
                .find(|s| s.started_at.is_some() && s.finished_at.is_none())
                .or_else(|| {
                    completed
                        .stages
                        .iter()
                        .rev()
                        .find(|s| s.started_at.is_some())
                })
                .ok_or("Late report has no admitted native stage")?;
            let target = self
                .stages
                .iter_mut()
                .find(|s| s.id == source.id)
                .ok_or("Late native report names another stage")?;
            if target.started_at.is_none() || target.started_at != source.started_at {
                return Err("Late native report differs from its admitted stage");
            }
            if let Some(existing) = &target.response {
                if existing.clone().into_report()? != report {
                    return Err("Native stage report is already accounted with different values");
                }
                return Ok(false);
            }
            target.response = Some(report.clone());
            self.response = Some(report);
            self.validate()?;
            return Ok(true);
        }
        if let Some(existing) = &self.response {
            if existing != &report {
                return Err("Stopped native report is already accounted with different values");
            }
            return Ok(false);
        }
        if let Some(stage) = self.stages.iter_mut().find(|s| s.finished_at.is_none()) {
            stage.response = Some(report.clone());
        }
        self.response = Some(report);
        self.validate()?;
        Ok(true)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        self.brief.validate()?;
        if self.id != self.brief.intent
            || self.parent.model_id.is_empty()
            || self.parent_connection_id.is_empty()
            || self.work_version.len() != 64
            || self.reason.as_ref().is_some_and(|r| r.len() > 2048)
        {
            return Err("Invalid assignment decision");
        }
        if matches!(
            self.phase,
            DecisionPhase::Selected | DecisionPhase::Fallback
        ) && self.selected.is_none()
        {
            return Err("Selected decision is missing its exact pair");
        }
        if self.phase == DecisionPhase::Prepared
            && (self.request.is_some() || self.response.is_some() || self.selected.is_some())
            || self.phase == DecisionPhase::Pending
                && (self.request.is_none() || self.selected.is_some())
        {
            return Err("Unsettled decision has inconsistent evidence");
        }
        if matches!(self.phase, DecisionPhase::Pending | DecisionPhase::Stopped) {
            if self.selected.is_some() {
                return Err("Pending or stopped native work cannot select a worker");
            }
            if let Some(report) = &self.response {
                if self.request.is_none()
                    || report.choice.is_some()
                    || report.confidence.is_some()
                    || !report.probabilities.is_empty()
                {
                    return Err(
                        "Pending or stopped native work retains only safe accounting and provenance",
                    );
                }
                report.validate(&[])?;
            }
        }
        if self.phase == DecisionPhase::Selected {
            let request = self
                .request
                .as_ref()
                .ok_or("Selected decision is missing its request")?;
            let response = self
                .response
                .as_ref()
                .ok_or("Selected decision is missing its response")?;
            request.validate_response(response)?;
            let final_response = self
                .stages
                .last()
                .and_then(|stage| stage.response.as_ref())
                .unwrap_or(response);
            if !self.stages.is_empty()
                && (self
                    .stages
                    .last()
                    .is_none_or(|stage| stage.failure.is_some() || stage.finished_at.is_none())
                    || final_response != response)
            {
                return Err("Partial hierarchy is not a final decision");
            }
            let chosen = final_response
                .choice
                .as_ref()
                .and_then(|key| request.candidates.iter().find(|c| &c.key == key));
            if chosen.is_none() || chosen != self.selected.as_ref() {
                return Err("Selected pair differs from the validated native choice");
            }
        }
        if self.legacy {
            if !self.stages.is_empty()
                || self
                    .request
                    .as_ref()
                    .is_some_and(|r| !r.evidence.is_empty() || r.candidates.len() > 254)
            {
                return Err("Legacy decisions cannot acquire complete-pool evidence or stages");
            }
        } else {
            if self.phase == DecisionPhase::Stopped
                && self
                    .stages
                    .iter()
                    .any(|stage| stage.finished_at.is_none() && stage.failure.is_none())
            {
                return Err("Stopped complete-pool work is missing a required stage Stop outcome");
            }
            match self.phase {
                DecisionPhase::Prepared | DecisionPhase::Pending => {
                    if self.finished_at.is_some() || self.reason.is_some() {
                        return Err("Unsettled complete-pool lifecycle has terminal facts");
                    }
                }
                _ => {
                    if self.finished_at.is_none()
                        || self.reason.as_ref().is_none_or(|r| r.is_empty())
                    {
                        return Err("Terminal complete-pool lifecycle facts are missing");
                    }
                }
            }
            if self.request.is_none() && (self.response.is_some() || self.selected.is_some()) {
                return Err("Complete-pool response or selection requires its captured authority");
            }
            if let Some(request) = &self.request {
                let mut pairs: Vec<&DecisionCandidate> = Vec::new();
                for candidate in &request.candidates {
                    if pairs.iter().any(|prior| {
                        prior.connection_id == candidate.connection_id
                            && prior.selection == candidate.selection
                    }) {
                        return Err("Complete pool contains duplicate immutable pairs");
                    }
                    pairs.push(candidate);
                }
                if self.stages.is_empty() && self.phase != DecisionPhase::Unsupported {
                    return Err("Complete-pool stages are missing");
                }
                if request.evidence.len() != request.candidates.len()
                    || request.evidence.iter().any(|(_, facts)| {
                        ["catalog", "capability", "pricing", "benchmark"]
                            .iter()
                            .any(|category| !facts.iter().any(|fact| &fact.category == category))
                    })
                {
                    return Err(
                        "Complete-pool evidence requires explicit facts or unknown observations for every candidate and category",
                    );
                }
            }
        }
        if let Some(request) = &self.request {
            request.validate_pool(self.brief.context_tokens)?;
            self.validate_stages(request)?;
        } else if !self.stages.is_empty() {
            return Err("Stages require an immutable complete request");
        }
        if let Some(selected) = &self.selected {
            selected.validate(self.brief.context_tokens)?;
            if self.phase == DecisionPhase::Fallback
                && (selected.selection != self.parent
                    || selected.connection_id != self.parent_connection_id)
            {
                return Err("Fallback differs from the actual parent snapshot");
            }
        }
        Ok(())
    }

    pub fn validate_transition(&self, next: &Self) -> Result<(), &'static str> {
        if !self.same_intent(next)
            || self.parent != next.parent
            || self.parent_connection_id != next.parent_connection_id
            || self.parent_revision != next.parent_revision
            || self.parent_run_id != next.parent_run_id
            || self.revision != next.revision
            || self.legacy != next.legacy
            || self.phase != DecisionPhase::Prepared && self.phase != DecisionPhase::Pending
            || self.phase == DecisionPhase::Pending && self.request != next.request
            || next.phase == DecisionPhase::Prepared
        {
            return Err("Immutable or terminal decision changed");
        }
        if self.stages.len() > next.stages.len() {
            return Err("Native stages cannot be discarded");
        }
        for (prior, next) in self.stages.iter().zip(&next.stages) {
            if !prior.same_input(next)
                || prior.started_at.is_some() && prior.started_at != next.started_at
                || prior.finished_at.is_some() && prior != next
            {
                return Err("Native stage input or completed outcome changed");
            }
        }
        next.validate()
    }

    fn validate_stages(&self, request: &DecisionRequest) -> Result<(), &'static str> {
        if self
            .stages
            .iter()
            .filter(|s| s.started_at.is_some() && s.finished_at.is_none() && s.failure.is_none())
            .count()
            > 1
        {
            return Err("Serial native hierarchy has competing active stage claims");
        }
        let mut ids = std::collections::HashSet::new();
        let mut expected: std::collections::HashSet<&str> =
            request.candidates.iter().map(|c| c.key.as_str()).collect();
        let mut level = 0;
        let mut index = 0;
        while index < self.stages.len() {
            let start = index;
            let mut members = std::collections::HashSet::new();
            let mut winners = std::collections::HashSet::new();
            let mut complete = true;
            while index < self.stages.len() && self.stages[index].level == level {
                let stage = &self.stages[index];
                stage.validate(request)?;
                if !ids.insert(&stage.id)
                    || stage.members.iter().any(|k| !members.insert(k.as_str()))
                {
                    return Err("Duplicate stage identity or level membership");
                }
                if stage.finished_at.is_some() && stage.failure.is_none() {
                    winners.insert(
                        stage
                            .response
                            .as_ref()
                            .and_then(|r| r.choice.as_deref())
                            .ok_or("Stage winner missing")?,
                    );
                } else {
                    complete = false;
                }
                index += 1;
            }
            if start == index || members != expected {
                return Err("Hierarchy omitted or substituted a required candidate");
            }
            if index < self.stages.len() && (!complete || winners.len() >= expected.len()) {
                return Err(
                    "Hierarchy continued after a failed stage or without reducing the pool",
                );
            }
            if self.phase == DecisionPhase::Selected
                && index == self.stages.len()
                && (!complete || index - start != 1)
            {
                return Err("Hierarchy has no complete final stage");
            }
            expected = winners;
            level += 1;
        }
        Ok(())
    }
}
