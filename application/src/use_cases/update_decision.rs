mod dependencies;
mod error;
mod output;

use crate::decision_access;
use crate::error_context::ErrorContext;
use crate::error_context::ResultExt;
use crate::error_context::diagnostic;
use crate::model_access;
use tokio_util::sync::CancellationToken;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    stop: bool,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    let mut decision = d
        .read_decision
        .call(id)
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Assignment intent was not found"))?;
    if stop
        && matches!(
            decision.phase,
            domain::DecisionPhase::Prepared | domain::DecisionPhase::Pending
        )
    {
        decision = stop_snapshot(decision).await?;
        decision.reason = Some(
            "Explicit Stop; attempted usage may be unknown. No fallback or child launch.".into(),
        );
        return save(&d, decision).await;
    }
    if decision.phase != domain::DecisionPhase::Prepared {
        let validation = revalidate(&d, &decision).await;
        return Ok(match validation {
            Ok(()) => Output {
                decision,
                blocked: None,
                diagnostics: Vec::new(),
            },
            Err(error) => blocked(decision, error),
        });
    }
    authority(&d, &decision).await.map_err(Error::caused_by)?;
    let id = match d
        .read_decider
        .call()
        .await
        .context("Configuration required: the native Decider preference could not be read")
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return Ok(blocked(
                decision,
                ErrorContext::caused_by(
                    "Configuration required: select a native Decider with /decider",
                    "No native Decider preference is saved",
                )
                .into(),
            ));
        }
        Err(error) => return Ok(blocked(decision, error.into())),
    };
    let (model, connection) = match decision_access::native(
        &d.read_models,
        &d.read_model_allowlist,
        &d.read_provider_connection,
        &id,
    )
    .await
    {
        Ok(native) => native,
        Err(error) => return Ok(blocked(decision, error)),
    };

    let mut candidates = Vec::new();
    for model in d.read_models.call().await.map_err(Error::caused_by)? {
        if model.validate_worker().is_err() {
            continue;
        }
        let efforts = if model.efforts.is_empty() {
            vec![None]
        } else {
            model.efforts.iter().copied().map(Some).collect()
        };
        for effort in efforts {
            let selection = domain::ModelSelection {
                provider_id: model.provider_id.clone(),
                model_id: model.id.clone(),
                effort,
            };
            if let Ok(candidate) = decision_access::candidate(
                &d.read_models,
                &d.read_model_allowlist,
                &d.read_provider_connection,
                &d.create_model_step_factory,
                &selection,
                decision.brief.context_tokens,
            )
            .await
            {
                candidates.push(candidate);
            }
        }
    }
    if candidates.is_empty() {
        decision.phase = domain::DecisionPhase::Configuration;
        decision.reason = Some("No eligible worker pairs: inspect /auth, positive /model allowlist, chat/tool/context/effort capabilities. Cancel preserves this intent; no default.".into());
        return save(&d, decision).await;
    }
    if candidates.len() > 254 {
        decision.phase = domain::DecisionPhase::Unsupported;
        decision.reason = Some(format!(
            "Unsupported complete pool: {} pairs plus abstention exceeds 255 options. No truncation or inference; hierarchical selection is not implemented.",
            candidates.len()
        ));
        return save(&d, decision).await;
    }
    let captured_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(Error::caused_by)?
        .as_secs();
    decision.request = Some(domain::DecisionRequest {
        model,
        connection_id: connection.id,
        candidates,
        captured_at,
        policy: domain::DecisionRequest::POLICY.into(),
    });
    decision.phase = domain::DecisionPhase::Pending;
    let revision = decision.revision;
    if cancellation.is_cancelled() {
        return Err(Error::caused_by("Stopped before native admission"));
    }
    let (prior_intent, checkpoint) = snapshot(decision).await?;
    decision = match d.update_decision.call(checkpoint, Some(revision)).await {
        Ok(saved) => saved,
        Err(error) => {
            let existing = d
                .read_decision
                .call(prior_intent.id.clone())
                .await
                .map_err(Error::caused_by)?;
            if let Some(existing) = existing
                && existing.same_intent(&prior_intent)
                && existing.phase != domain::DecisionPhase::Prepared
            {
                let validation = revalidate(&d, &existing).await;
                let mut output = match validation {
                    Ok(()) => Output {
                        decision: existing,
                        blocked: None,
                        diagnostics: Vec::new(),
                    },
                    Err(error) => blocked(existing, error),
                };
                output.diagnostics.push(Error::caused_by(error));
                return Ok(output);
            }
            return Err(Error::caused_by(error));
        }
    };

    let (mut decision, native_input) = snapshot(decision).await?;
    let native_cancellation = cancellation.child_token();
    let native = d
        .create_native_decision
        .call(native_input, native_cancellation.clone());
    tokio::pin!(native);
    let response = loop {
        tokio::select! {
            biased;
            result = &mut native => break result,
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                let control = d.read_decision.call(decision.id.clone()).await.map_err(Error::caused_by)?;
                if control.as_ref().is_some_and(|v| v.phase == domain::DecisionPhase::Stopped) {
                    native_cancellation.cancel();
                    match native.await {
                        Ok(response) => return Ok(Output {
                            decision: checkpoint_report(&d, decision, response).await?,
                            blocked: None,
                            diagnostics: Vec::new(),
                        }),
                        Err(mut error) => {
                            let decision = if let Some(report) = error.reported_response.take() {
                                checkpoint_report(&d, decision, *report).await?
                            } else {
                                control.ok_or_else(|| Error::caused_by("Stopped intent disappeared"))?
                            };
                            return Ok(Output {
                                decision,
                                blocked: None,
                                diagnostics: vec![Error::caused_by(error)],
                            });
                        }
                    }
                }
            }
        }
    };
    let mut diagnostics = Vec::new();
    let received = match response {
        Ok(response) => {
            decision = checkpoint_report(&d, decision, response).await?;
            true
        }
        Err(mut error) => {
            if let Some(report) = error.reported_response.take() {
                decision = checkpoint_report(&d, decision, *report).await?;
            }
            let is_unsupported = unsupported(&error);
            diagnostics.push(Error::caused_by(ErrorContext::caused_by(
                if is_unsupported {
                    "Unsupported complete native input"
                } else {
                    "Native request failed; exact-parent recovery only"
                },
                error,
            )));
            if is_unsupported {
                decision.phase = domain::DecisionPhase::Unsupported;
                decision.reason = Some("Unsupported complete native input exceeds the conservative byte envelope. No truncation, fallback or inference; hierarchical selection is not implemented.".into());
                return save_with_diagnostics(&d, decision, diagnostics).await;
            }
            false
        }
    };
    if decision.phase == domain::DecisionPhase::Stopped {
        return Ok(Output {
            decision,
            blocked: None,
            diagnostics,
        });
    }
    if cancellation.is_cancelled() {
        decision = stop_snapshot(decision).await?;
        decision.reason = Some(
            "Explicit Stop; native usage is unknown unless reported. No fallback or child launch."
                .into(),
        );
        return save_with_diagnostics(&d, decision, diagnostics).await;
    }
    if let Err(error) = authority(&d, &decision).await {
        decision.phase = domain::DecisionPhase::Configuration;
        decision.reason = Some("Approved assignment requires inspection; no replacement".into());
        diagnostics.push(Error::caused_by(ErrorContext::caused_by(
            "Approved assignment requires inspection",
            error,
        )));
        return save_with_diagnostics(&d, decision, diagnostics).await;
    }
    let request = decision
        .request
        .as_ref()
        .ok_or_else(|| Error::caused_by("Native request snapshot missing"))?;
    let current_policy = d
        .read_model_allowlist
        .call()
        .await
        .map_err(Error::caused_by)?;
    if !current_policy.permits(&request.model.provider_id, &request.model.id) {
        decision.phase = domain::DecisionPhase::Configuration;
        decision.reason = Some("Selected native Decider was disallowed by the user. Open /model settings; no replacement or fallback.".into());
        return save_with_diagnostics(&d, decision, diagnostics).await;
    }
    let mut failure = if decision.response.is_some() {
        "Native answer failed; reported accounting retained. Exact eligible parent recovery only."
    } else {
        "Native request failed; usage is unknown. Exact eligible parent recovery only."
    }
    .to_owned();
    if received {
        let (restored, validation, candidate) = tokio::task::spawn_blocking(move || {
            let request = decision
                .request
                .as_ref()
                .ok_or_else(|| Error::caused_by("Native request snapshot missing"))?;
            let response = decision
                .response
                .as_ref()
                .ok_or_else(|| Error::caused_by("Native response missing"))?;
            let validation = request.validate_response(response);
            let candidate = response
                .choice
                .as_ref()
                .and_then(|key| request.candidates.iter().find(|c| &c.key == key))
                .cloned();
            Ok::<_, Error>((decision, validation, candidate))
        })
        .await
        .map_err(Error::caused_by)??;
        decision = restored;
        if let Err(error) = validation {
            failure = error.into();
            diagnostics.push(Error::caused_by(ErrorContext::caused_by(
                "Native response validation failed",
                error,
            )));
        } else if let Some(candidate) = candidate {
            if !current_policy.permits(
                &candidate.selection.provider_id,
                &candidate.selection.model_id,
            ) {
                decision.phase = domain::DecisionPhase::Configuration;
                decision.selected = Some(candidate);
                decision.reason = Some("Selected worker was disallowed by the user. Open /model settings; no replacement.".into());
                return save_with_diagnostics(&d, decision, diagnostics).await;
            }
            match current(&d, &decision, &candidate).await {
                Ok(()) => {
                    decision.selected = Some(candidate);
                    decision.phase = domain::DecisionPhase::Selected;
                    decision.reason = Some("Validated native opaque choice under suitability-first, cost-conscious policy. No worker has been launched; confidence is not correctness probability.".into());
                    return save_with_diagnostics(&d, decision, diagnostics).await;
                }
                Err(error) if model_access::is_disallowed(error.as_ref()) => {
                    decision.phase = domain::DecisionPhase::Configuration;
                    decision.selected = Some(candidate);
                    decision.reason = Some("Selected worker was disallowed by the user. Open /model settings; no replacement.".into());
                    diagnostics.push(Error::caused_by(error));
                    return save_with_diagnostics(&d, decision, diagnostics).await;
                }
                Err(error) => {
                    failure = "Native choice became stale or incompatible before acceptance".into();
                    diagnostics.push(Error::caused_by(ErrorContext::caused_by(
                        "Native choice became stale or incompatible before acceptance",
                        error,
                    )));
                }
            }
        } else {
            failure = "Native refusal or explicit abstention".into();
        }
    }

    match decision_access::candidate(
        &d.read_models,
        &d.read_model_allowlist,
        &d.read_provider_connection,
        &d.create_model_step_factory,
        &decision.parent,
        decision.brief.context_tokens,
    )
    .await
    {
        Ok(candidate) if candidate.connection_id == decision.parent_connection_id => {
            decision.selected = Some(candidate);
            decision.phase = domain::DecisionPhase::Fallback;
            decision.reason = Some(format!(
                "{failure}. Retained the exact actual-parent model AND thinking; no worker launched."
            ));
        }
        result => {
            let cause: Box<dyn std::error::Error + Send + Sync> = match result {
                Err(error) => error,
                Ok(_) => "Exact parent connection snapshot changed".into(),
            };
            diagnostics.push(Error::caused_by(ErrorContext::caused_by(
                "Exact parent snapshot is ineligible; configure /auth or /model",
                cause,
            )));
            decision.phase = domain::DecisionPhase::Configuration;
            decision.reason = Some(format!(
                "{failure}. Exact parent snapshot is ineligible; configure /auth or /model. No compiled default or effort rewrite."
            ));
        }
    }
    save_with_diagnostics(&d, decision, diagnostics).await
}

fn unsupported(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if error
            .downcast_ref::<std::io::Error>()
            .is_some_and(|e| e.kind() == std::io::ErrorKind::Unsupported)
        {
            return true;
        }
        current = error.source();
    }
    false
}

async fn authority(
    d: &Dependencies,
    decision: &domain::AssignmentDecision,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parent = d.read_conversation.call(decision.parent_id.clone()).await?;
    let assignment = parent
        .assignment
        .as_ref()
        .ok_or("Actual CoS assignment is missing")?;
    if parent.resume_required
        || parent.agent.as_ref().is_none_or(|a| {
            a.id.0 != decision.parent_agent_id || a.role != domain::AgentRole::ChiefOfStaff
        })
        || assignment.work_id != decision.work_id
        || assignment.intent != decision.work_intent
        || assignment.proposal.version != decision.work_version
    {
        return Err("Actual approved CoS identity or Resume gate changed".into());
    }
    let work = d
        .read_work
        .call(decision.work_id.clone())
        .await?
        .ok_or("Approved work is missing")?;
    work.authorize_assignment(assignment, &decision.parent_id)?;
    Ok(())
}

async fn current(
    d: &Dependencies,
    decision: &domain::AssignmentDecision,
    selected: &domain::DecisionCandidate,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let current = decision_access::candidate(
        &d.read_models,
        &d.read_model_allowlist,
        &d.read_provider_connection,
        &d.create_model_step_factory,
        &selected.selection,
        decision.brief.context_tokens,
    )
    .await?;
    if current.connection_id != selected.connection_id
        || current.model.canonical_id != selected.model.canonical_id
    {
        return Err("Selected worker connection or canonical revision changed".into());
    }
    Ok(())
}

async fn revalidate(
    d: &Dependencies,
    decision: &domain::AssignmentDecision,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    authority(d, decision).await?;
    if let Some(selected) = &decision.selected {
        current(d, decision, selected).await?;
    }
    Ok(())
}

async fn stop_snapshot(
    mut decision: domain::AssignmentDecision,
) -> Result<domain::AssignmentDecision, Error> {
    tokio::task::spawn_blocking(move || {
        decision.stop().map_err(Error::caused_by)?;
        Ok(decision)
    })
    .await
    .map_err(Error::caused_by)?
}

async fn snapshot(
    decision: domain::AssignmentDecision,
) -> Result<(domain::AssignmentDecision, domain::AssignmentDecision), Error> {
    tokio::task::spawn_blocking(move || (decision.clone(), decision))
        .await
        .map_err(Error::caused_by)
}

async fn checkpoint_report(
    d: &Dependencies,
    decision: domain::AssignmentDecision,
    response: domain::DecisionResponse,
) -> Result<domain::AssignmentDecision, Error> {
    let (decision, response) = tokio::task::spawn_blocking(move || {
        let mut decision = decision;
        decision.response = Some(response.clone().into_report().map_err(Error::caused_by)?);
        Ok::<_, Error>((decision, response))
    })
    .await
    .map_err(Error::caused_by)??;
    let revision = decision.revision;
    let mut saved = d
        .update_decision
        .call(decision, Some(revision))
        .await
        .map_err(Error::caused_by)?;
    if saved.phase != domain::DecisionPhase::Stopped {
        saved.response = Some(response);
    }
    Ok(saved)
}

async fn save(d: &Dependencies, decision: domain::AssignmentDecision) -> Result<Output, Error> {
    save_with_diagnostics(d, decision, Vec::new()).await
}

async fn save_with_diagnostics(
    d: &Dependencies,
    mut decision: domain::AssignmentDecision,
    diagnostics: Vec<Error>,
) -> Result<Output, Error> {
    decision.finished_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(Error::caused_by)?
            .as_secs(),
    );
    let revision = decision.revision;
    let decision = d
        .update_decision
        .call(decision, Some(revision))
        .await
        .map_err(Error::caused_by)?;

    let blocked = (decision.phase == domain::DecisionPhase::Configuration)
        .then(|| decision.reason.clone().unwrap_or_default());
    Ok(Output {
        decision,
        blocked,
        diagnostics,
    })
}

fn blocked(
    decision: domain::AssignmentDecision,
    error: Box<dyn std::error::Error + Send + Sync>,
) -> Output {
    Output {
        blocked: Some(diagnostic(error.as_ref())),
        decision,
        diagnostics: vec![Error::caused_by(error)],
    }
}
