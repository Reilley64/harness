use crate::error_context::ResultExt;
use crate::ports::ReadWork;
pub async fn read(
    read: &ReadWork,
    conversation: &domain::Conversation,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(a) = &conversation.assignment {
        let work = read
            .call(a.work_id.clone())
            .await?
            .ok_or("Approved work is unavailable")?;
        let approval = work.authorize(&a.intent, &a.proposal.version).context("Approved work changed or authority is missing; inspect and approve the current scope/criteria version before new work")?;
        if work.creator_id != a.creator_id
            || approval.coordinator_id != conversation.id
            || approval.approved_at != a.approved_at
            || work.current()? != &a.proposal
        {
            return Err("Approved handoff no longer matches backend authority".into());
        }
    }
    Ok(())
}
