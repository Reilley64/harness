mod record;
use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::instructions;
use application::ports::PortFuture;
use std::path::Path;
type Error = Box<dyn std::error::Error + Send + Sync>;
pub fn create_coordinator(
    directory: &Path,
) -> impl Fn(
    domain::ApprovedWork,
    domain::WorkApproval,
    domain::Model,
    domain::ProviderConnection,
    domain::InstructionSnapshot,
    bool,
) -> PortFuture<Result<String, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |work, approval, model, connection, instruction, adopt_default| {
        let directory = directory.clone();
        Box::pin(async move {
            work.authorize(&approval.intent, &approval.version)?;
            if approval.selection.provider_id != model.provider_id
                || approval.selection.model_id != model.id
                || connection.provider_id != model.provider_id
            {
                return Err("Coordinator selection differs from assignment intent".into());
            }
            model.validate_effort(approval.selection.effort)?;
            instructions::validate(&instruction)?;
            if instruction.phase != "coordination" {
                return Err("Coordinator requires actual CoS instructions".into());
            }
            let id = approval.coordinator_id.clone();
            let _guard = conversations::lock(&directory, &id).await?;
            let proposal = work.current()?.clone();
            let assignment = domain::WorkAssignment {
                work_id: work.id,
                intent: approval.intent,
                creator_id: work.creator_id,
                proposal,
                approved_at: approval.approved_at,
            };
            match tokio::fs::symlink_metadata(directory.join(format!("{id}.json"))).await {
                Ok(_) => {
                    let bytes = conversations::read(&directory, &id).await?;
                    let expected = id.clone();
                    return tokio::task::spawn_blocking(move || {
                        let existing: domain::Conversation =
                            serde_json::from_slice::<Record>(&bytes)?.try_into()?;
                        if existing.id != expected
                            || existing.assignment.as_ref() != Some(&assignment)
                        {
                            return Err(
                                "Coordinator identity is already bound to different work".into()
                            );
                        }
                        Ok(expected)
                    })
                    .await?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
            let now = conversations::now()?;
            let conversation = domain::Conversation {
                id: id.clone(),
                agent: Some(domain::Agent {
                    id: domain::AgentId(id.clone()),
                    role: domain::AgentRole::ChiefOfStaff,
                }),
                assignment: Some(assignment),
                research: None,

                workspace: Some(work.workspace),
                provider_connection_id: connection.id,
                model,
                effort: approval.selection.effort,

                instruction_snapshots: vec![instruction.clone()],
                selected_instructions: adopt_default.then_some(instruction.id),

                messages: Vec::new(),
                runs: Vec::new(),
                current_work: None,
                resume_required: false,

                created_at: now,
                updated_at: now,
                revision: 0,
            };
            let bytes = tokio::task::spawn_blocking(move || {
                let r = Record::from(conversation);
                let c: domain::Conversation = r.try_into()?;
                encode_bounded(&Record::from(c), 16 * 1024 * 1024)
            })
            .await??;
            conversations::write(&directory, &id, &bytes).await?;
            Ok(id)
        })
    }
}
