use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to retain private execution diagnostics")]
pub struct CreateExecutionDiagnosticsError;

port! { pub CreateExecutionDiagnostics(parent_id:String,execution_id:String,run_id:Option<String>,diagnostics:Vec<Box<dyn std::error::Error+Send+Sync>>,) -> PortFuture<Result<(),CreateExecutionDiagnosticsError>>; }
