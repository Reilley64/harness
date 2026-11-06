use crate::tool_io;
use application::ports::PortFuture;
pub fn read_tools(
    enabled: bool,
) -> impl Fn() -> PortFuture<Result<Vec<domain::Tool>, Box<dyn std::error::Error + Send + Sync>>>
+ Send
+ Sync
+ use<> {
    move || {
        Box::pin(async move {
            if !enabled {
                return Ok(Vec::new());
            }
            let mut names = domain::Tool::ALL
                .into_iter()
                .filter(|t| *t != domain::Tool::AstSearch)
                .collect::<Vec<_>>();
            let mut command = tokio::process::Command::new("ast-grep");
            tool_io::process_environment(&mut command);
            if let Ok(Ok(output)) = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                command.arg("--version").kill_on_drop(true).output(),
            )
            .await
                && output.status.success()
                && String::from_utf8_lossy(&output.stdout).trim() == "ast-grep 0.51.0"
            {
                names.push(domain::Tool::AstSearch);
            }
            Ok(names)
        })
    }
}
