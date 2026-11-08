use application::error_context::ResultExt;
use application::error_context::diagnostic;
use infrastructure::dependencies::Dependencies;
use infrastructure::settings::Settings;
use ohno::AppError;

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
        .read_children(|_| (&dependencies).into())
        .update_children(|_| (&dependencies).into())
        .read_decision(|_| (&dependencies).into())
        .update_decision(|_| (&dependencies).into())
        .read_deciders(|_| (&dependencies).into())
        .update_decider(|_| (&dependencies).into())
        .read_work(|_| (&dependencies).into())
        .update_work(|_| (&dependencies).into())
        .authentication(|_| (&dependencies).into())
        .models(|_| (&dependencies).into())
        .create_agent(|_| (&dependencies).into())
        .read_agent(|_| (&dependencies).into())
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

    presentation_acp::run(presentation).await?;
    Ok(())
}
