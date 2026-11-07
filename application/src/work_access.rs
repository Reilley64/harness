use crate::error_context::ResultExt;
use crate::ports::ReadWork;
pub async fn read(
    read: &ReadWork,
    conversation_id: &str,
    assignment: Option<&domain::WorkAssignment>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(a) = assignment {
        let work = read
            .call(a.work_id.clone())
            .await?
            .ok_or("Approved work is unavailable")?;
        work.authorize_assignment(a, conversation_id).context("Approved work changed or authority is missing; inspect and approve the current scope/criteria version before new work")?;
    }
    Ok(())
}
