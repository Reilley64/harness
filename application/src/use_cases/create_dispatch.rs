mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::child_runtime::StartError;
use crate::decision_access;
use crate::error_context;
use crate::ports::PortFuture;
use crate::ports::PresentTurnEvent;
use crate::research_access;
use crate::use_cases::create_message;
use crate::use_cases::read_children;
use crate::work_access;
use tokio_util::sync::CancellationToken;

pub fn call(
    dependencies: impl Into<Dependencies>,
    parent_id: String,
    parent_revision: u64,
    brief: domain::AssignmentBrief,
    routed: bool,
    cancellation: CancellationToken,
) -> PortFuture<Result<Output, Error>> {
    let dependencies = dependencies.into();
    Box::pin(dispatch(
        dependencies,
        parent_id,
        parent_revision,
        brief,
        routed,
        cancellation,
    ))
}

async fn dispatch(
    dependencies: Dependencies,
    parent_id: String,
    parent_revision: u64,
    brief: domain::AssignmentBrief,
    routed: bool,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.worker;
    brief.validate().map_err(Error::caused_by)?;
    if brief.role != "research" {
        return Err(Error::caused_by(
            "Only Research dispatch is available; Implementers require managed workspaces",
        ));
    }
    let intent = uuid::Uuid::parse_str(&brief.intent).map_err(Error::caused_by)?;
    if intent.to_string() != brief.intent {
        return Err(Error::caused_by("Research intent must be a canonical UUID"));
    }
    let (prior, mut diagnostics) = read_children::load(
        &read_children::Dependencies {
            read_coordination: d.read_coordination.clone(),
            read_conversation: d.read_conversation.clone(),
            read_execution_claims: d.read_execution_claims.clone(),
            update_coordination: d.update_coordination.clone(),
        },
        parent_id.clone(),
    )
    .await
    .map_err(Error::caused_by)?;
    if let Some(execution) = prior.as_ref().and_then(|coordination| {
        coordination
            .children
            .iter()
            .find(|child| child.intent == brief.intent)
    }) {
        if execution.brief != brief || execution.routed != routed {
            return Err(error::retain(
                Error::caused_by("Dispatch key conflicts with its accepted exact payload"),
                diagnostics,
            ));
        }
        return Ok(Output {
            diagnostics,
            execution: execution.clone(),
            accepted: false,
        });
    }
    if !diagnostics.is_empty() {
        return Err(error::retain(
            Error::caused_by(
                "Required coordination was reconciled as Unknown; inspect before new work",
            ),
            diagnostics,
        ));
    }
    if let Some(mut execution) = d
        .read_execution_claim
        .call(brief.intent.clone(), cancellation.clone())
        .await
        .map_err(Error::caused_by)?
    {
        if !execution.matches_request(&parent_id, &brief, routed) {
            return Err(Error::caused_by(
                "Dispatch intent is already claimed by a different exact typed payload",
            ));
        }
        execution.status = domain::ChildExecutionStatus::Unknown;
        execution.outcome = Some(domain::ChildOutcome {
            id: execution.execution_id.clone(),
            status: domain::ChildExecutionStatus::Unknown,
            conversation_id: execution.agent_id.clone(),
            run_id: None,
            reply_id: None,
            summary: "An immutable execution claim exists without its recorded parent admission. The accepted identities, pair and guidance were retained. Effects are not proven unsent; inspect and explicitly reconcile. No child was replayed.".into(),
            artifacts: Vec::new(),
        });
        let response = d
            .update_coordination
            .call(
                parent_id.clone(),
                domain::CoordinationCommand::RetainClaim(Box::new(execution)),
                prior.as_ref().map(|state| state.revision),
                CancellationToken::new(),
            )
            .await
            .map_err(Error::caused_by)?;
        let execution = response
            .coordination
            .children
            .into_iter()
            .find(|child| child.intent == brief.intent)
            .ok_or_else(|| Error::caused_by("Retained execution is missing"))?;
        return Ok(Output {
            diagnostics,
            execution,
            accepted: false,
        });
    }
    let parent = d
        .read_conversation
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    let agent = parent
        .agent
        .as_ref()
        .filter(|agent| {
            matches!(
                agent.role,
                domain::AgentRole::Griller | domain::AgentRole::ChiefOfStaff
            )
        })
        .ok_or_else(|| {
            Error::caused_by("Research must be dispatched by its actual Griller or Chief of Staff")
        })?;
    if parent.revision != parent_revision || parent.resume_required {
        return Err(Error::caused_by(
            "Parent changed or requires explicit Resume before dispatch",
        ));
    }
    let run = parent
        .runs
        .last()
        .filter(|run| run.status == domain::RunStatus::Running)
        .ok_or_else(|| {
            Error::caused_by("Research dispatch requires the actual active parent run")
        })?;
    work_access::read(&d.read_work, &parent_id, parent.assignment.as_ref())
        .await
        .map_err(Error::caused_by)?;
    let actual_parent = domain::ModelSelection {
        provider_id: parent.model.provider_id.clone(),
        model_id: parent.model.id.clone(),
        effort: parent.effort,
    };
    let (mut selected, captured_parent, captured_connection, captured_revision) = if routed {
        let decision = d
            .read_decision
            .call(brief.intent.clone())
            .await
            .map_err(Error::caused_by)?
            .ok_or_else(|| Error::caused_by("Prepared native selection is missing"))?;
        if decision.parent_id != parent_id
            || decision.parent_agent_id != agent.id.0
            || decision.brief != brief
            || !matches!(
                decision.phase,
                domain::DecisionPhase::Selected | domain::DecisionPhase::Fallback
            )
            || parent.assignment.as_ref().is_none_or(|assignment| {
                decision.work_id != assignment.work_id
                    || decision.work_intent != assignment.intent
                    || decision.work_version != assignment.proposal.version
            })
        {
            return Err(Error::caused_by(
                "Native intent is unsettled, stopped or differs from actual approved work",
            ));
        }
        decision.validate().map_err(Error::caused_by)?;
        (
            decision
                .selected
                .ok_or_else(|| Error::caused_by("Native selection has no worker pair"))?,
            decision.parent,
            decision.parent_connection_id,
            decision.parent_revision,
        )
    } else {
        if agent.role != domain::AgentRole::Griller {
            return Err(Error::caused_by(
                "Chief of Staff Research requires its prepared native selection",
            ));
        }
        (
            decision_access::candidate(
                &d.read_models,
                &d.read_model_allowlist,
                &d.read_provider_connection,
                &d.create_model_step_factory,
                &actual_parent,
                brief.context_tokens,
            )
            .await
            .map_err(Error::caused_by)?,
            actual_parent,
            parent.provider_connection_id.clone(),
            parent_revision,
        )
    };
    if !routed {
        selected.key = uuid::Uuid::new_v5(&intent, b"inherited-pair").to_string();
    }
    let current = decision_access::candidate(
        &d.read_models,
        &d.read_model_allowlist,
        &d.read_provider_connection,
        &d.create_model_step_factory,
        &selected.selection,
        brief.context_tokens,
    )
    .await
    .map_err(Error::caused_by)?;
    if current.connection_id != selected.connection_id
        || current.model.canonical_id != selected.model.canonical_id
    {
        return Err(Error::caused_by(
            "Selected Research connection or canonical identity changed; configure without replacement",
        ));
    }
    let workspace = parent
        .workspace
        .clone()
        .ok_or_else(|| Error::caused_by("Research parent has no saved workspace"))?;
    let adopted = d
        .read_research_guidance
        .call(brief.intent.clone())
        .await
        .map_err(Error::caused_by)?;
    let instructions = if let Some(adopted) = adopted {
        if !adopted.matches(&parent) {
            return Err(Error::caused_by(
                "Research adoption differs from the actual parent or approved work; inspect without replacement",
            ));
        }
        adopted.snapshot
    } else {
        let guidance = d
            .read_instructions
            .call(workspace.clone(), "research".into(), Vec::new(), Vec::new())
            .await
            .map_err(Error::caused_by)?;
        if !guidance.overrides.is_empty() {
            return Err(Error::caused_by(error_context::ErrorContext::caused_by(
                "Research overrides require explicit inspection and adoption for this intent in /children guidance.",
                "Research guidance is not adopted",
            )));
        }
        guidance.available.ok_or_else(|| {
            Error::caused_by("Required Research instructions are unavailable; inspect guidance")
        })?
    };
    let instructions = tokio::task::spawn_blocking(move || {
        instructions.validate()?;
        Ok::<_, &'static str>(instructions)
    })
    .await
    .map_err(Error::caused_by)?
    .map_err(Error::caused_by)?;
    let current_parent = d
        .read_conversation
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    if current_parent.revision != parent_revision || current_parent.resume_required {
        return Err(Error::caused_by(
            "Parent changed while preparing Research; no child started",
        ));
    }
    work_access::read(&d.read_work, &parent_id, current_parent.assignment.as_ref())
        .await
        .map_err(Error::caused_by)?;
    if cancellation.is_cancelled() {
        return Err(Error::caused_by(
            "Research dispatch stopped before execution admission",
        ));
    }
    let id = |name: &[u8]| uuid::Uuid::new_v5(&intent, name).to_string();
    let execution = domain::ChildExecution {
        intent: brief.intent.clone(),
        assignment_id: id(b"assignment"),
        agent_id: id(b"agent"),
        execution_id: id(b"execution"),

        parent_id: parent_id.clone(),
        parent_agent_id: agent.id.0.clone(),
        parent_run_id: run.id.clone(),
        parent_revision: captured_revision,

        parent_selection: captured_parent,
        parent_connection_id: captured_connection,

        work_id: parent
            .assignment
            .as_ref()
            .map(|assignment| assignment.work_id.clone()),
        work_intent: parent
            .assignment
            .as_ref()
            .map(|assignment| assignment.intent.clone()),
        work_version: parent
            .assignment
            .as_ref()
            .map(|assignment| assignment.proposal.version.clone()),
        approved_at: parent
            .assignment
            .as_ref()
            .map(|assignment| assignment.approved_at),

        brief,
        selected,
        instructions,
        routed,

        controller: uuid::Uuid::new_v4().to_string(),
        status: domain::ChildExecutionStatus::Held,
        stop_requested: false,

        outcome: None,

        delivered_to: None,
        follow_up_completed_by: None,
        history: Vec::new(),
    };
    let response = d
        .update_coordination
        .call(
            parent_id,
            domain::CoordinationCommand::Dispatch(Box::new(execution.clone())),
            prior.map(|coordination| coordination.revision),
            cancellation.clone(),
        )
        .await
        .map_err(Error::caused_by)?;
    let accepted = response
        .coordination
        .children
        .into_iter()
        .find(|child| child.intent == execution.intent)
        .ok_or_else(|| Error::caused_by("Accepted Research execution is missing"))?;
    if !response.claimed {
        return Ok(Output {
            diagnostics,
            execution: accepted,
            accepted: false,
        });
    }
    if cancellation.is_cancelled() {
        return stopped_before_spawn(&d,accepted,diagnostics,response.coordination.revision,"Dispatch stopped after durable admission and before a child future was created. No child inference or tools started.").await;
    }
    let worker = d.clone();
    let child = accepted.clone();
    let token = CancellationToken::new();
    let execution_token = token.clone();
    let task = Box::pin(async move {
        let result = execute(worker.clone(), child.clone(), workspace, execution_token).await;
        if let Err(original) = result {
            let fenced = worker
                .update_coordination
                .call(
                    child.parent_id,
                    domain::CoordinationCommand::MarkUnknown {
                        execution_id: child.execution_id,
                        controller: child.controller,
                    },
                    None,
                    CancellationToken::new(),
                )
                .await;
            let failure = match fenced {
                Ok(_) => original,
                Err(fence) => error::retain(original, vec![Box::new(fence)]),
            };
            return Err(Box::new(failure) as Box<dyn std::error::Error + Send + Sync>);
        }
        Ok(())
    });
    let started = d.child_runtime.start(
        accepted.parent_id.clone(),
        accepted.execution_id.clone(),
        task,
        token,
    );
    if let Err(error) = started {
        if matches!(error, StartError::Closing) {
            diagnostics.push(Box::new(error));
            return stopped_before_spawn(&d,accepted,diagnostics,response.coordination.revision,"Runtime shutdown won admission after the durable claim. No child inference or tools started.").await;
        }
        return Err(Error::caused_by(error));
    }
    Ok(Output {
        diagnostics,
        execution: accepted,
        accepted: true,
    })
}

async fn stopped_before_spawn(
    d: &create_message::Dependencies,
    accepted: domain::ChildExecution,
    mut diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
    revision: u64,
    summary: &str,
) -> Result<Output, Error> {
    let receipt = domain::ChildOutcome {
        id: accepted.execution_id.clone(),
        status: domain::ChildExecutionStatus::Stopped,
        conversation_id: accepted.agent_id.clone(),
        run_id: None,
        reply_id: None,
        summary: summary.into(),
        artifacts: Vec::new(),
    };
    let response = d
        .update_coordination
        .call(
            accepted.parent_id.clone(),
            domain::CoordinationCommand::Receive {
                execution_id: accepted.execution_id.clone(),
                controller: accepted.controller.clone(),
                outcome: receipt,
            },
            Some(revision),
            CancellationToken::new(),
        )
        .await
        .map_err(|failure| {
            error::retain(Error::caused_by(failure), std::mem::take(&mut diagnostics))
        })?;
    let execution = response
        .coordination
        .children
        .into_iter()
        .find(|child| child.intent == accepted.intent)
        .ok_or_else(|| {
            error::retain(
                Error::caused_by("Stopped accepted execution is missing"),
                std::mem::take(&mut diagnostics),
            )
        })?;
    Ok(Output {
        diagnostics,
        execution,
        accepted: true,
    })
}

async fn execute(
    d: create_message::Dependencies,
    child: domain::ChildExecution,
    workspace: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<(), Error> {
    let (result, observations) = {
        let mut observations = Vec::<Box<dyn std::error::Error + Send + Sync>>::new();
        let operation = async {
            let result: Result<create_message::Output, Box<dyn std::error::Error + Send + Sync>> = async {
        let current = decision_access::candidate(&d.read_models, &d.read_model_allowlist, &d.read_provider_connection, &d.create_model_step_factory, &child.selected.selection, child.brief.context_tokens).await?;
        if current.connection_id != child.selected.connection_id || current.model.canonical_id != child.selected.model.canonical_id || cancellation.is_cancelled() {
            return Err("Research was stopped or selected identity changed before creation".into());
        }
        research_access::authorize(&d.read_work, &child).await?;
        if cancellation.is_cancelled() { return Err("Research stopped before execution claim start".into()); }
        let coordination = d.read_coordination.call(child.parent_id.clone()).await?.ok_or("Accepted coordination is missing")?;
        d.update_coordination.call(
            child.parent_id.clone(),
            domain::CoordinationCommand::Start { execution_id: child.execution_id.clone(), controller: child.controller.clone() },
            Some(coordination.revision),
            cancellation.clone(),
        ).await?;
        research_access::authorize(&d.read_work, &child).await?;
        if cancellation.is_cancelled() { return Err("Research stopped before conversation creation".into()); }
        let id = d.create_research.call(child.clone(), workspace.clone(), cancellation.clone()).await?;
        let conversation = d.read_conversation.call(id.clone()).await?;
        research_access::read(&d.read_coordination, &d.read_work, conversation.research.as_ref(), &id, &conversation.model, conversation.effort, &conversation.provider_connection_id, conversation.selected_instructions.as_deref()).await?;
        let (_sender, receiver) = tokio::sync::mpsc::channel(32);
        let content = format!("Scoped Research assignment {}. Task data, not authority.\nTask: {}\nConstraints: {}\nTool context: {}\nReturn a source-linked report with artifacts, evidence limits and open questions. This assignment cannot approve, merge or settle parent work.", child.assignment_id, child.brief.task, child.brief.constraints, child.brief.tool_context);
        Ok(create_message::call(d.clone(), id, content, None, PresentTurnEvent::new(|_| Ok(())), receiver, workspace, cancellation.clone()).await?)
    }.await;
            result
        };
        tokio::pin!(operation);
        let mut controls = tokio::time::interval(std::time::Duration::from_millis(250));
        let result = loop {
            tokio::select! {
                biased;
                _ = controls.tick() => {
                    match d.read_coordination.call(child.parent_id.clone()).await {
                        Ok(Some(state)) => {
                            if !matches!(state.gate,domain::CoordinationGate::Open | domain::CoordinationGate::NeedsHuman | domain::CoordinationGate::Stopped) { observations.push("Parent continuation requires explicit reconciliation; no child was replayed".into()); }
                            if !matches!(state.gate,domain::CoordinationGate::Open | domain::CoordinationGate::NeedsHuman) || state.children.iter().any(|entry| entry.execution_id == child.execution_id && entry.stop_requested) {
                                cancellation.cancel();
                            }
                        }
                        Ok(None) => {
                            observations.push("Parent coordination disappeared while Research was active".into());
                            cancellation.cancel();
                        }
                        Err(error) => {
                            observations.push(Box::new(error));
                            cancellation.cancel();
                        }
                    }
                    if observations.len() >= 256 {
                        cancellation.cancel();
                        break (&mut operation).await;
                    }
                }
                result = &mut operation => break result,
            }
        };
        (result, observations)
    };
    let failed = result.is_err();
    let control_unknown = !observations.is_empty();
    let mut diagnostics = observations;
    let failure_summary = result
        .as_ref()
        .err()
        .map(|error| error_context::diagnostic(error.as_ref()));
    let conversation = match result {
        Ok(output) => {
            diagnostics.extend(output.tool_errors);
            Some(output.conversation)
        }
        Err(original) => {
            diagnostics.push(original);
            match d.read_conversation.call(child.agent_id.clone()).await {
                Ok(conversation) => Some(conversation),
                Err(recovery) => {
                    diagnostics.push(Box::new(recovery));
                    None
                }
            }
        }
    };
    let stopped = cancellation.is_cancelled();
    let retained = !diagnostics.is_empty();
    if retained {
        d.create_execution_diagnostics
            .call(
                child.parent_id.clone(),
                child.execution_id.clone(),
                std::mem::take(&mut diagnostics),
            )
            .await
            .map_err(Error::caused_by)?;
    }
    let outcome_id = child.execution_id.clone();
    let conversation_id = child.agent_id.clone();
    let outcome = tokio::task::spawn_blocking(move || {
        let reply = conversation.as_ref().and_then(|conversation| conversation.messages.last());
        let uncertain = conversation.as_ref().is_some_and(|conversation| conversation.messages.iter().flat_map(|message| &message.steps).flat_map(|step| &step.calls).any(|call| call.status == domain::ToolStatus::Unknown));
        let status = if uncertain || control_unknown {
            domain::ChildExecutionStatus::Unknown
        } else if !failed && reply.is_some_and(|reply| reply.status == domain::MessageStatus::Completed) {
            domain::ChildExecutionStatus::Completed
        } else if stopped {
            domain::ChildExecutionStatus::Stopped
        } else if failed {
            domain::ChildExecutionStatus::Failed
        } else {
            match reply.map(|reply| reply.status) {
                Some(domain::MessageStatus::Completed) => domain::ChildExecutionStatus::Completed,
                Some(domain::MessageStatus::Cancelled) => domain::ChildExecutionStatus::Stopped,
                _ => domain::ChildExecutionStatus::NeedsHuman,
            }
        };
        let summary = failure_summary.unwrap_or_else(|| {
            let text = reply.map_or("", |reply| reply.content.as_str());
            if text.len() <= 8192 { text.to_owned() } else { "Report exceeds the bounded summary; inspect the source-linked child conversation and its recorded artifacts.".into() }
        });
        domain::ChildOutcome {
            id: outcome_id.clone(),
            status,
            conversation_id: conversation_id.clone(),
            run_id: conversation.as_ref().and_then(|conversation| conversation.runs.last()).map(|run| run.id.clone()),
            reply_id: reply.map(|reply| reply.id.clone()),
            summary,
            artifacts: if retained {
                vec![format!("harness://conversations/{conversation_id}"),format!("harness://execution-diagnostics/{outcome_id}")]
            } else {
                vec![format!("harness://conversations/{conversation_id}")]
            },
        }
    }).await.map_err(|error| error::retain(Error::caused_by(error), std::mem::take(&mut diagnostics)))?;
    let coordination = d
        .read_coordination
        .call(child.parent_id.clone())
        .await
        .map_err(|error| error::retain(Error::caused_by(error), std::mem::take(&mut diagnostics)))?
        .ok_or_else(|| {
            error::retain(
                Error::caused_by(
                    "Accepted coordination disappeared; retain execution for explicit inspection",
                ),
                std::mem::take(&mut diagnostics),
            )
        })?;
    d.update_coordination
        .call(
            child.parent_id,
            domain::CoordinationCommand::Receive {
                execution_id: child.execution_id,
                controller: child.controller,
                outcome,
            },
            Some(coordination.revision),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| {
            error::retain(Error::caused_by(error), std::mem::take(&mut diagnostics))
        })?;
    Ok(())
}
