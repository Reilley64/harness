use super::Dependencies;
use super::Error;
use crate::coordination_recovery;
use crate::execution_ancestry;
use crate::ports::UpdateCoordinationError;

pub(super) async fn validate(
    d: &Dependencies,
    conversation: &domain::Conversation,
    request_run: &str,
    approval_id: Option<&str>,
) -> Result<(), Error> {
    let Some(research) = &conversation.research else {
        return Ok(());
    };
    let boundaries = execution_ancestry::read(
        &d.read_coordination,
        &d.read_conversation,
        &conversation.id,
        research,
    )
    .await
    .map_err(Error::caused_by)?;
    if boundaries.iter().any(|boundary| {
        boundary
            .scope
            .as_ref()
            .is_some_and(|scope| scope.approved_by.is_none())
    }) {
        return Err(Error::caused_by(
            "Answer cannot bypass an ancestor's pending scope approval",
        ));
    }
    let mut owner = d
        .read_coordination
        .call(research.parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Answer requires retained execution ownership"))?;
    let command = domain::CoordinationCommand::AnswerChild {
        execution_id: research.execution_id.clone(),
        request_run: request_run.into(),
        approval_id: approval_id.map(str::to_owned),
    };
    tokio::task::spawn_blocking(move || owner.apply(command))
        .await
        .map_err(Error::caused_by)?
        .map(|_| ())
        .map_err(Error::caused_by)
}

pub(super) async fn reconcile(
    d: &Dependencies,
    conversation: domain::Conversation,
    research: &domain::ResearchAssignment,
    request_run: &str,
    original: UpdateCoordinationError,
    diagnostics: &mut Vec<Box<dyn std::error::Error + Send + Sync>>,
) -> Result<domain::Conversation, Error> {
    let owner = match d.read_coordination.call(research.parent_id.clone()).await {
        Ok(Some(owner)) => owner,
        Ok(None) => {
            return Err(super::error::retain(
                Error::caused_by(original),
                vec![Box::new(Error::caused_by(
                    "Answer acknowledgement lost its execution ownership",
                ))],
            ));
        }
        Err(read) => {
            return Err(super::error::retain(
                Error::caused_by(original),
                vec![Box::new(read)],
            ));
        }
    };
    if owner.authority_complete
        && owner.agent_id == research.parent_agent_id
        && owner.conversation_id == research.parent_id
        && owner.children.iter().any(|child| {
            child.execution_id == research.execution_id
                && child.agent_id == conversation.id
                && child.assignment_id == research.assignment_id
                && child.intent == research.intent
                && child.outcome.is_none()
                && child
                    .history
                    .last()
                    .is_some_and(|receipt| receipt.outcome.run_id.as_deref() == Some(request_run))
        })
    {
        diagnostics.push(Box::new(original));
        return Ok(conversation);
    }

    match coordination_recovery::restore_answer(
        &d.read_coordination,
        &d.update_coordination,
        &d.update_conversation,
        conversation,
    )
    .await
    {
        Ok(_) => Err(Error::caused_by(original)),
        Err(recovery) => Err(super::error::retain(
            Error::caused_by(original),
            vec![recovery],
        )),
    }
}
