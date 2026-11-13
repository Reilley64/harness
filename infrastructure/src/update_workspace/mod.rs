mod arguments;
pub(crate) use arguments::decode;
pub(crate) use arguments::definition;
mod git;
mod record;

use self::record::CheckRecord;
use self::record::Configuration;
use self::record::Record;
use self::record::SubmissionRecord;
use crate::cache::encode_bounded;
use crate::create_process;
use crate::storage;
use application::error_context::ResultExt;
use application::ports::PortFuture;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn update_workspace(
    app_data_path: &Path,
) -> impl Fn(
    domain::ChildExecution,
    PathBuf,
    domain::WorkspaceAction,
    CancellationToken,
) -> PortFuture<Result<domain::AssignmentWorkspace, Error>>
+ Send
+ Sync
+ use<> {
    let root = app_data_path.join("workspaces");
    move |execution, repository, action, cancellation| {
        let root = root.clone();
        Box::pin(async move {
            execution.validate()?;
            if execution.brief.role != "implementer" || !execution.routed {
                return Err("Managed workspaces require a routed approved Implementer".into());
            }

            let id = uuid::Uuid::parse_str(&execution.assignment_id)?;
            if id.to_string() != execution.assignment_id {
                return Err("Workspace assignment identity must be canonical".into());
            }
            let directory = root.join(id.to_string());

            storage::private_directory(&directory).await?;
            let _operation = storage::lock(&directory.join("operation")).await?;
            let record_path = directory.join("workspace.json");
            let artifact = format!("harness://workspaces/{id}");
            let repository = tokio::fs::canonicalize(repository).await?;
            let mut record = match tokio::fs::symlink_metadata(&record_path).await {
                Ok(_) => {
                    let bytes = storage::read_private(&record_path, 4 * 1024 * 1024).await?;
                    tokio::task::spawn_blocking(move || serde_json::from_slice::<Record>(&bytes))
                        .await??
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    if action != domain::WorkspaceAction::Prepare {
                        return Err(
                            "Workspace intent is missing; no Git operation was replayed".into()
                        );
                    }
                    let text = execution.brief.tool_context.clone();
                    let configuration = tokio::task::spawn_blocking(move || {
                        serde_json::from_str::<Configuration>(&text)
                    })
                    .await??;
                    validate_configuration(&configuration)?;
                    git::call(
                        &repository,
                        &["check-ref-format", &configuration.integration],
                        None,
                        &cancellation,
                    )
                    .await?;
                    let work_id = execution
                        .work_id
                        .as_ref()
                        .ok_or("Approved workspace work identity is missing")?;
                    let work_uuid = uuid::Uuid::parse_str(work_id)?;
                    let spec_path = root.join("specs").join(format!("{work_uuid}.json"));
                    {
                        let _spec = storage::lock(&spec_path).await?;
                        let bytes = encode_bounded(
                            &(repository.clone(), configuration.integration.clone()),
                            16_384,
                        )?;
                        match tokio::fs::symlink_metadata(&spec_path).await {
                            Ok(_) => {
                                if storage::read_private(&spec_path, 16_384).await? != bytes {
                                    return Err("This approved spec already owns a different repository or integration branch".into());
                                }
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                storage::replace(&spec_path, &bytes).await?
                            }
                            Err(error) => return Err(error.into()),
                        }
                    }
                    let base = git::call(
                        &repository,
                        &[
                            "rev-parse",
                            "--verify",
                            &format!("{}^{{commit}}", configuration.integration),
                        ],
                        None,
                        &cancellation,
                    )
                    .await?;
                    let cycles = domain::ImplementationCycles::default();
                    let record = Record {
                        format: 1,
                        assignment_id: execution.assignment_id.clone(),
                        execution_id: execution.execution_id.clone(),
                        parent_id: execution.parent_id.clone(),
                        agent_id: execution.agent_id.clone(),
                        work_id: work_id.clone(),
                        work_version: execution
                            .work_version
                            .clone()
                            .ok_or("Workspace approval version is missing")?,
                        repository: repository.clone(),
                        path: directory.join("checkout"),
                        branch: format!("refs/heads/harness/{work_uuid}/{id}"),
                        marker: format!("refs/harness/assignments/{id}"),
                        base: base.clone(),
                        tip: base,
                        configuration,
                        phase: "planned".into(),
                        submissions: Vec::new(),
                        cycle_allowance: cycles.allowance,
                        rework_runs: cycles.rework_runs,
                    };
                    save(&record_path, &record).await?;
                    record
                }
                Err(error) => return Err(error.into()),
            };

            if record.format != 1
                || record.assignment_id != execution.assignment_id
                || record.execution_id != execution.execution_id
                || record.parent_id != execution.parent_id
                || record.agent_id != execution.agent_id
                || Some(&record.work_id) != execution.work_id.as_ref()
                || Some(&record.work_version) != execution.work_version.as_ref()
                || record.repository != repository
                || record.path != directory.join("checkout")
                || record.branch != format!("refs/heads/harness/{}/{id}", record.work_id)
                || record.marker != format!("refs/harness/assignments/{id}")
                || record.submissions.len() > 64
                || [&record.base, &record.tip].iter().any(|hash| {
                    !matches!(hash.len(), 40 | 64)
                        || !hash
                            .bytes()
                            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                })
            {
                return Err("Workspace ownership differs from its immutable accepted claim".into());
            }
            let mut cycles = domain::ImplementationCycles {
                allowance: record.cycle_allowance,
                rework_runs: record.rework_runs.clone(),
            };
            cycles.validate()?;
            validate_configuration(&record.configuration)?;

            if !matches!(
                record.phase.as_str(),
                "planned" | "refs_pending" | "tree_pending" | "ready" | "checking" | "submitted"
            ) || record.submissions.iter().any(|submission| {
                !matches!(submission.commit.len(), 40 | 64)
                    || !submission
                        .commit
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    || submission.results.len() > record.configuration.checks.len()
                    || submission
                        .results
                        .iter()
                        .zip(&record.configuration.checks)
                        .any(|(result, command)| {
                            &result.command != command
                                || domain::ToolStatus::parse(&result.status).is_none()
                        })
                    || submission.complete
                        && (submission.results.len() != record.configuration.checks.len()
                            || submission
                                .results
                                .iter()
                                .any(|result| result.status != "succeeded"))
            }) {
                return Err("Workspace check evidence is missing or inconsistent".into());
            }
            if record
                .submissions
                .last()
                .is_some_and(|submission| submission.commit != record.tip)
                || record.phase == "submitted"
                    && !record
                        .submissions
                        .last()
                        .is_some_and(|submission| submission.complete)
                || record.phase == "checking"
                    && !record
                        .submissions
                        .last()
                        .is_some_and(|submission| !submission.complete)
            {
                return Err(
                    "Workspace submission does not match its exact recorded check revision".into(),
                );
            }

            if record.configuration
                != serde_json::from_str::<Configuration>(&execution.brief.tool_context)?
            {
                return Err("Workspace payload differs from the exact accepted intent".into());
            }

            match action {
                domain::WorkspaceAction::Prepare => {}
                domain::WorkspaceAction::Inspect => {
                    verify(&record, &cancellation).await?;
                    if git::call(&record.path, &["rev-parse", "HEAD"], None, &cancellation).await?
                        != record.tip
                    {
                        return Err("Workspace moved since the recorded submission; retained checks are stale".into());
                    }
                }
                domain::WorkspaceAction::Open { resume, rework } => {
                    if let Some(run) = rework {
                        uuid::Uuid::parse_str(&run)?;
                        let admitted = cycles.admit(run).map_err(|error| -> Error { error.into() })
                            .context("The three implementation cycles are exhausted. Preserve this assignment and request a human extension; a new intent or routing cannot reset the allowance.")?;
                        if admitted {
                            record.rework_runs = cycles.rework_runs;
                            save(&record_path, &record).await?;
                        }
                    }

                    if record.phase == "planned" {
                        record.phase = "refs_pending".into();
                        save(&record_path, &record).await?;
                        let transaction = format!(
                            "start\ncreate {} {}\ncreate {} {}\nprepare\ncommit\n",
                            record.branch, record.base, record.marker, record.base
                        );
                        git::call(
                            &repository,
                            &["update-ref", "--stdin"],
                            Some(&transaction),
                            &cancellation,
                        )
                        .await?;
                        record.phase = "tree_pending".into();
                        save(&record_path, &record).await?;
                    } else if !matches!(record.phase.as_str(), "ready" | "submitted") {
                        if !resume {
                            return Err("Managed Git or checks have an uncertain result; inspect native facts and explicitly Resume, never resend".into());
                        }
                        if tokio::fs::symlink_metadata(&record.path).await.is_ok() {
                            verify(&record, &cancellation).await?;
                        } else {
                            verify_refs(&record, &cancellation).await?;
                        }
                        record.phase = "ready".into();
                        save(&record_path, &record).await?;
                    }

                    let registered = git::call(
                        &repository,
                        &["worktree", "list", "--porcelain"],
                        None,
                        &cancellation,
                    )
                    .await?;
                    if tokio::fs::symlink_metadata(&record.path).await.is_err() {
                        if registered
                            .lines()
                            .any(|line| line == format!("worktree {}", record.path.display()))
                        {
                            return Err("Workspace is absent but still registered; explicit Git reconciliation is required".into());
                        }
                        verify_refs(&record, &cancellation).await?;
                        record.phase = "tree_pending".into();
                        save(&record_path, &record).await?;
                        git::call(
                            &repository,
                            &[
                                "worktree",
                                "add",
                                "--",
                                record.path.to_str().ok_or("Workspace path is not UTF-8")?,
                                record
                                    .branch
                                    .strip_prefix("refs/heads/")
                                    .ok_or("Invalid assignment branch")?,
                            ],
                            None,
                            &cancellation,
                        )
                        .await?;
                    }
                    verify(&record, &cancellation).await?;

                    for path in &record.configuration.setup {
                        let path = record.path.join(path);
                        let resolved = tokio::fs::canonicalize(&path).await.context("Required workspace setup is missing. Inspect the owned workspace's required setup references; no ignored fixture or secret was copied.")?;
                        if !resolved.starts_with(&record.path) {
                            return Err("Required setup escapes the owned workspace; no fallback is permitted".into());
                        }
                    }

                    record.phase = "ready".into();
                    save(&record_path, &record).await?;
                }
                domain::WorkspaceAction::Submit { commit } => {
                    if !matches!(record.phase.as_str(), "ready" | "submitted") {
                        return Err("Submission requires reconciled workspace effects; uncertain checks are not replayed".into());
                    }
                    verify(&record, &cancellation).await?;
                    let tip = git::call(
                        &record.path,
                        &["rev-parse", "--verify", "HEAD"],
                        None,
                        &cancellation,
                    )
                    .await?;
                    if commit != tip {
                        return Err("Submission must identify the exact full native HEAD commit, not a moving ref or stale candidate".into());
                    }
                    if record
                        .submissions
                        .last()
                        .is_some_and(|s| s.complete && s.commit == commit)
                    {
                        return Ok(record.projection(artifact));
                    }
                    if record.submissions.len() >= 64 {
                        return Err("Submission history is full; evidence cannot be dropped".into());
                    }
                    git::call(
                        &record.path,
                        &["merge-base", "--is-ancestor", &record.base, &tip],
                        None,
                        &cancellation,
                    )
                    .await?;

                    record.tip = tip.clone();
                    record.phase = "checking".into();
                    record.submissions.push(SubmissionRecord {
                        commit: tip.clone(),
                        results: Vec::new(),
                        complete: false,
                    });
                    save(&record_path, &record).await?;
                    let execute = create_process::create_process(&root);
                    for command in record.configuration.checks.clone() {
                        let result = execute(
                            record.path.clone(),
                            domain::CreateProcessArguments {
                                command: command.clone(),
                                timeout_seconds: None,
                            },
                            cancellation.clone(),
                        )
                        .await?;
                        let passed =
                            result.status == domain::ToolStatus::Succeeded && !result.truncated;
                        record
                            .submissions
                            .last_mut()
                            .ok_or("Missing submission intent")?
                            .results
                            .push(CheckRecord {
                                command,
                                status: result.status.as_str().into(),
                                output: result.output,
                            });
                        save(&record_path, &record).await?;
                        if !passed {
                            record.phase = "ready".into();
                            save(&record_path, &record).await?;
                            return Err("Required native check failed, was cancelled or exceeded retained output; inspect the durable submission evidence".into());
                        }
                    }

                    verify(&record, &cancellation).await?;
                    if git::call(&record.path, &["rev-parse", "HEAD"], None, &cancellation).await?
                        != tip
                    {
                        return Err(
                            "Candidate moved during checks; check evidence does not qualify it"
                                .into(),
                        );
                    }

                    record
                        .submissions
                        .last_mut()
                        .ok_or("Missing submission intent")?
                        .complete = true;
                    record.phase = "submitted".into();
                    save(&record_path, &record).await?;
                }
            }

            Ok(record.projection(artifact))
        })
    }
}

fn validate_configuration(configuration: &Configuration) -> Result<(), Error> {
    if !configuration.integration.starts_with("refs/heads/")
        || configuration.integration.contains(['\n', '\0', ' '])
        || configuration.integration.len() > 256
        || configuration.setup.is_empty()
        || configuration.setup.len() > 32
        || configuration.setup.iter().any(|path| {
            path.len() > 4096
                || !Path::new(path)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
        })
        || configuration.checks.is_empty()
        || configuration.checks.len() > 16
        || configuration.checks.iter().any(|command| {
            command.trim().is_empty() || command.len() > 4096 || command.contains('\0')
        })
    {
        return Err("Implementer toolContext requires an integration refs/heads branch, relative setup references and nonempty bounded required checks".into());
    }
    Ok(())
}

async fn save(path: &Path, record: &Record) -> Result<(), Error> {
    let record = record.clone();
    let bytes =
        tokio::task::spawn_blocking(move || encode_bounded(&record, 4 * 1024 * 1024)).await??;
    storage::replace(path, &bytes).await?;
    Ok(())
}

async fn verify_refs(record: &Record, cancellation: &CancellationToken) -> Result<(), Error> {
    if git::call(
        &record.repository,
        &["rev-parse", "--verify", &record.marker],
        None,
        cancellation,
    )
    .await?
        != record.base
    {
        return Err("Native assignment ownership marker changed or is foreign".into());
    }
    if git::call(
        &record.repository,
        &["rev-parse", "--verify", &record.branch],
        None,
        cancellation,
    )
    .await?
        != record.tip
    {
        return Err(
            "Assignment branch advanced outside its recorded checkpoint; inspect before recreating"
                .into(),
        );
    }
    Ok(())
}

async fn verify(record: &Record, cancellation: &CancellationToken) -> Result<(), Error> {
    if git::call(
        &record.repository,
        &["rev-parse", "--verify", &record.marker],
        None,
        cancellation,
    )
    .await?
        != record.base
    {
        return Err("Native assignment ownership marker changed or is foreign".into());
    }
    if git::call(&record.path, &["symbolic-ref", "HEAD"], None, cancellation).await?
        != record.branch
    {
        return Err("Workspace is detached or has a foreign branch".into());
    }
    let common = git::call(
        &record.path,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        None,
        cancellation,
    )
    .await?;
    let source = git::call(
        &record.repository,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        None,
        cancellation,
    )
    .await?;
    if common != source
        || !git::call(
            &record.path,
            &["status", "--porcelain=v1", "--untracked-files=all"],
            None,
            cancellation,
        )
        .await?
        .is_empty()
    {
        return Err(
            "Workspace is foreign or dirty; exact committed submissions cannot use it".into(),
        );
    }
    Ok(())
}
