mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(dependencies: impl Into<Dependencies>) -> Result<Vec<Output>, Error> {
    let dependencies = dependencies.into();
    let models = dependencies
        .read_models
        .call()
        .await
        .map_err(Error::caused_by)?;
    Ok(models.into_iter().map(Output::from).collect())
}
