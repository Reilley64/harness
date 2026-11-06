mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(dependencies: impl Into<Dependencies>) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let selected_model = dependencies
        .read_model_selection
        .call()
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { selected_model })
}
