use application::error_context::ResultExt;
use application::error_context::diagnostic;
use infrastructure::dependencies::Dependencies;
use infrastructure::settings::Settings;
use ohno::AppError;

#[ohno::error]
#[display("ACP and owned execution drain both failed: {drain}")]
struct ShutdownError {
    drain: application::use_cases::delete_executions::Error,
}

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|_| {
        eprintln!("Harness encountered an unexpected internal failure.");
    }));
    if let Err(error) = run().await {
        eprintln!("{}", diagnostic(error.as_ref()));
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let dependencies =
        Dependencies::build(Settings::load().context("Could not load Harness configuration")?);

    let presentation = presentation_acp::Dependencies::builder()
        .mcp(|_| (&dependencies).into())
        .providers(|_| (&dependencies).into())
        .create_dispatch(|_| {
            application::use_cases::create_dispatch::Dependencies::new((&dependencies).into())
        })
        .update_research_guidance(|_| {
            application::use_cases::update_research_guidance::Dependencies::new(
                (&dependencies).into(),
            )
        })
        .update_workspace(|_| {
            application::use_cases::update_workspace::Dependencies::new((&dependencies).into())
        })
        .create_instruction(|_| (&dependencies).into())
        .create_scope(|_| (&dependencies).into())
        .update_scope(|_| {
            application::use_cases::update_scope::Dependencies::new((&dependencies).into())
        })
        .delete_execution(|_| (&dependencies).into())
        .delete_executions(|_| (&dependencies).into())
        .read_children(|_| (&dependencies).into())
        .update_children(|_| (&dependencies).into())
        .read_decision(|_| (&dependencies).into())
        .update_decision(|_| (&dependencies).into())
        .update_routing_evidence(|_| (&dependencies).into())
        .read_deciders(|_| (&dependencies).into())
        .update_decider(|_| (&dependencies).into())
        .read_usage(|_| (&dependencies).into())
        .update_provider_retry(|_| (&dependencies).into())
        .read_work(|_| (&dependencies).into())
        .update_work(|_| (&dependencies).into())
        .authentication(|_| (&dependencies).into())
        .models(|_| (&dependencies).into())
        .create_agent(|_| (&dependencies).into())
        .read_agent(|_| (&dependencies).into())
        .read_creation_map(|_| (&dependencies).into())
        .read_setup(|_| (&dependencies).into())
        .update_setup(|_| (&dependencies).into())
        .read_guidance(|_| (&dependencies).into())
        .update_guidance(|_| (&dependencies).into())
        .update_agent(|_| (&dependencies).into())
        .create_conversation(|_| (&dependencies).into())
        .read_conversation(|_| (&dependencies).into())
        .create_message(|_| (&dependencies).into())
        .read_model_allowlist(|_| (&dependencies).into())
        .update_model_allowlist(|_| (&dependencies).into())
        .read_model_selection(|_| (&dependencies).into())
        .update_model_selection(|_| (&dependencies).into())
        .build();

    let result = presentation_acp::run(presentation).await;
    let drained = application::use_cases::delete_executions::call(&dependencies).await;
    match (result, drained) {
        (Ok(()), Ok(_)) => Ok(()),
        (Err(error), Ok(_)) => Err(error.into()),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(original), Err(drain)) => Err(ShutdownError::caused_by(drain, original).into()),
    }
}
