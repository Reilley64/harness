mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: Option<String>,
    workspace: std::path::PathBuf,
    phase: Option<String>,
    scopes: Option<Vec<String>>,
    references: Option<Vec<String>>,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let conversation = if let Some(id) = id {
        Some(
            dependencies
                .read_conversation
                .call(id)
                .await
                .map_err(Error::caused_by)?,
        )
    } else {
        None
    };
    let selected = conversation
        .as_ref()
        .and_then(|conversation| conversation.selected_instructions())
        .cloned();
    let workspace = conversation
        .as_ref()
        .and_then(|conversation| conversation.workspace.clone())
        .unwrap_or(workspace);
    let resolution = dependencies
        .read_instructions
        .call(
            workspace,
            phase.unwrap_or_else(|| {
                selected
                    .as_ref()
                    .map_or("grilling".into(), |snapshot| snapshot.phase.clone())
            }),
            scopes.unwrap_or_else(|| {
                selected
                    .as_ref()
                    .map_or_else(Vec::new, |snapshot| snapshot.scopes.clone())
            }),
            references.unwrap_or_else(|| {
                selected
                    .as_ref()
                    .map_or_else(Vec::new, |snapshot| snapshot.references.clone())
            }),
        )
        .await
        .map_err(Error::caused_by)?;
    Ok(Output {
        revision: conversation
            .as_ref()
            .map(|conversation| conversation.revision),
        current: conversation
            .as_ref()
            .and_then(|conversation| conversation.run_instructions().ok())
            .cloned(),
        current_run: conversation
            .as_ref()
            .and_then(|conversation| conversation.runs.last())
            .map(|run| run.id.clone()),
        historical_unknown: conversation.as_ref().map_or_else(Vec::new, |conversation| {
            conversation
                .runs
                .iter()
                .filter(|run| run.instructions.is_none())
                .map(|run| run.id.clone())
                .collect()
        }),
        selected,
        resolution,
    })
}
