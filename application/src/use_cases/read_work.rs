mod dependencies;
mod error;
mod output;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;
pub async fn call(dependencies: impl Into<Dependencies>, id: String) -> Result<Output, Error> {
    Ok(Output {
        work: dependencies
            .into()
            .read_work
            .call(id)
            .await
            .map_err(Error::caused_by)?,
    })
}
