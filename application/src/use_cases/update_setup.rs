mod dependencies;
mod error;
mod output;
use crate::error_context;
use crate::error_context::ResultExt;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    revision: u64,
    proposal: String,
    confirm: bool,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let _guard = dependencies
        .lock_conversation
        .call(id.clone())
        .await
        .map_err(Error::caused_by)?;

    let mut conversation = dependencies
        .read_conversation
        .call(id)
        .await
        .map_err(Error::caused_by)?;
    if conversation.revision != revision {
        return Err("Stale Setup revision")
            .context("Setup changed; inspect the exact proposal again before confirmation")
            .map_err(Error::caused_by);
    }

    let mut call = conversation
        .setup_call(&proposal)
        .map_err(Error::caused_by)?;
    let draft = call
        .draft
        .clone()
        .ok_or_else(|| Error::caused_by("Exact Setup draft is missing"))?;
    if cancellation.is_cancelled() {
        return Ok(Output { conversation });
    }

    if confirm {
        call.start().map_err(Error::caused_by)?;
        conversation
            .update_setup_call(call.clone())
            .map_err(Error::caused_by)?;
        conversation = dependencies
            .update_conversation
            .call(conversation)
            .await
            .map_err(Error::caused_by)?;
        let result = dependencies
            .update_setup_draft
            .call(*draft, cancellation)
            .await
            .unwrap_or_else(|error| domain::ToolResult::failed(error_context::diagnostic(&error)));
        call.finish(result).map_err(Error::caused_by)?;
    } else {
        call.finish(domain::ToolResult {
            status: domain::ToolStatus::Skipped,
            output: "Human cancelled this proposal. No file was written.".into(),
            artifact: None,
            truncated: false,
        })
        .map_err(Error::caused_by)?;
    }

    call = dependencies
        .create_tool_output
        .call(conversation.id.clone(), call)
        .await
        .map_err(Error::caused_by)?;
    conversation
        .update_setup_call(call)
        .map_err(Error::caused_by)?;
    let conversation = dependencies
        .update_conversation
        .call(conversation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { conversation })
}
