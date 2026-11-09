mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use tokio_util::sync::CancellationToken;

pub async fn call(dependencies: impl Into<Dependencies>) -> Result<Output, Error> {
    let d = dependencies.into();
    let active = d.child_runtime.active();
    let stopped = active.len();
    let mut failures = Vec::<Box<dyn std::error::Error + Send + Sync>>::new();
    for (parent, execution) in active {
        let result = async {
            let state = d
                .read_coordination
                .call(parent.clone())
                .await?
                .ok_or("Required child coordination is missing during shutdown")?;
            d.update_coordination
                .call(
                    parent,
                    domain::CoordinationCommand::StopChild {
                        execution_id: execution.clone(),
                    },
                    Some(state.revision),
                    CancellationToken::new(),
                )
                .await?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        }
        .await;
        if let Err(error) = result {
            failures.push(error);
        }
        d.child_runtime.cancel(&execution);
    }
    failures.extend(d.child_runtime.drain().await);
    if failures.is_empty() {
        Ok(Output { stopped })
    } else {
        let original = failures.remove(0);
        Err(error::retain(Error::caused_by(original), failures))
    }
}
