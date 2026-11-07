mod bundle;

use crate::instructions;
use crate::repository;
use application::error_context::ResultExt;
use application::ports::PortFuture;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[ohno::error]
#[display("{message}")]
struct Failure {
    message: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    format: u32,
    root: Option<String>,
    #[serde(default)]
    overrides: Vec<Override>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Override {
    id: String,
    path: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    format: u32,
    version: String,
    body: String,
    requires: Vec<String>,
}

pub fn read_instructions(
    app_data_path: &Path,
) -> impl Fn(
    PathBuf,
    String,
    Vec<String>,
    Vec<String>,
) -> PortFuture<Result<domain::InstructionResolution, Error>>
+ Send
+ Sync
+ use<> {
    let local = app_data_path.join("instructions/config.json");
    move |workspace, phase, scopes, references| {
        let local = local.clone();
        Box::pin(async move {
            let bundled = bundle::read();
            let mut resolution = domain::InstructionResolution {
                available: None,
                blocked: Vec::new(),
                overrides: Vec::new(),
                bundled,
            };
            match resolve(
                &workspace,
                &local,
                phase,
                scopes,
                references,
                &mut resolution,
            )
            .await
            {
                Ok(snapshot) => resolution.available = Some(snapshot),
                Err(error) => resolution.blocked.push(error.to_string()),
            }
            Ok(resolution)
        })
    }
}

async fn resolve(
    workspace: &Path,
    local: &Path,
    phase: String,
    mut scopes: Vec<String>,
    mut references: Vec<String>,
    resolution: &mut domain::InstructionResolution,
) -> Result<domain::InstructionSnapshot, Error> {
    if !matches!(
        phase.as_str(),
        "grilling" | "specification" | "tickets" | "setup"
    ) {
        return Err("Required phase must be grilling, specification or tickets".into());
    }
    if scopes.len() > 32 || references.len() > 32 {
        return Err("Instruction context exceeds 32 scopes/references".into());
    }
    scopes.sort();
    scopes.dedup();
    references.sort();
    references.dedup();
    let (workspace, _) = repository::identity(workspace)
        .await
        .context("Instruction workspace is unavailable")?;
    let mut selected = BTreeMap::new();
    let mut root = None;
    let mut root_source = None;
    for path in [
        local.to_path_buf(),
        workspace.join(".harness/instructions/config.json"),
    ] {
        let Some(bytes) = read_optional(&path, 65_536).await.map_err(|error| {
            Failure::caused_by(
                format!(
                    "Required instruction configuration is unreadable: {}",
                    path.display()
                ),
                error,
            )
        })?
        else {
            continue;
        };
        let config: Configuration = parse(bytes).await.map_err(|error| {
            Failure::caused_by(
                format!("Invalid instruction configuration: {}", path.display()),
                error,
            )
        })?;
        if config.format != 1 || config.overrides.len() > 64 {
            return Err("Incompatible instruction configuration format or size".into());
        }
        if let Some(value) = config.root {
            resolution
                .overrides
                .push(format!("Root selection from {}: {value}", path.display()));
            root = Some(value);
            root_source = Some(path.clone());
        }
        let mut ids = BTreeSet::new();
        for entry in config.overrides {
            if !ids.insert(entry.id.clone()) {
                return Err("Duplicate instruction override identity".into());
            }
            if !resolution
                .bundled
                .iter()
                .any(|instruction| instruction.id == entry.id)
            {
                return Err(format!("Unknown instruction override: {}", entry.id).into());
            }
            let source = path
                .parent()
                .ok_or("Instruction configuration has no directory")?
                .join(relative(&entry.path)?);
            if let Some(previous) = selected.insert(entry.id.clone(), source.clone()) {
                resolution.overrides.push(format!(
                    "{}: {} shadows {}",
                    entry.id,
                    source.display(),
                    previous.display()
                ));
            } else {
                resolution.overrides.push(format!(
                    "{}: {} overrides bundled default",
                    entry.id,
                    source.display()
                ));
            }
        }
    }
    let mut catalog: BTreeMap<_, _> = resolution
        .bundled
        .iter()
        .map(|instruction| (instruction.id.clone(), instruction.clone()))
        .collect();
    let mut needed = vec![
        format!(
            "role/{}",
            if phase == "setup" { "setup" } else { "griller" }
        ),
        format!("procedure/{phase}"),
    ];
    let mut loaded = BTreeMap::new();
    while let Some(id) = needed.pop() {
        if loaded.contains_key(&id) {
            continue;
        }
        let mut instruction = catalog
            .remove(&id)
            .ok_or_else(|| format!("Missing required instruction dependency: {id}"))?;
        if let Some(path) = selected.get(&id) {
            let bytes = read_required(path, 262_144).await.map_err(|error| {
                Failure::caused_by(
                    format!(
                        "Required instruction override is unavailable: {}",
                        path.display()
                    ),
                    error,
                )
            })?;
            let response: Response = parse(bytes).await.map_err(|error| {
                Failure::caused_by(
                    format!("Invalid required instruction override: {}", path.display()),
                    error,
                )
            })?;
            if response.format != 1 {
                return Err(format!("Incompatible required instruction override: {id}").into());
            }
            if instruction
                .requires
                .iter()
                .any(|required| !response.requires.contains(required))
            {
                return Err(
                    format!("Incompatible required dependency list in override: {id}").into(),
                );
            }
            instruction.version = response.version;
            instruction.source = path.to_string_lossy().into();
            instruction.upstream = Vec::new();
            instruction.body = response.body;
            instruction.requires = response.requires;
        }
        needed.extend(instruction.requires.iter().cloned());
        loaded.insert(id, instruction);
        if loaded.len() > 64 || needed.len() > 2048 {
            return Err("Required instruction dependency closure exceeds its bound".into());
        }
    }
    let mut documents = BTreeMap::new();
    if phase == "setup" {
        if let Some(root) = &root {
            let selection = domain::Instruction {
                id: "repository/setup-root-selection".into(),
                version: "captured-content".into(),
                source: root_source
                    .as_ref()
                    .ok_or("Root selection has no source")?
                    .to_string_lossy()
                    .into(),
                upstream: Vec::new(),
                requires: Vec::new(),
                body: format!(
                    "Explicit intended root: {root}. Preserve this selection; a missing file is a Setup bootstrap omission, not permission to choose another root."
                ),
            };
            documents.insert(selection.source.clone(), selection);
        }
        for name in ["AGENTS.md", "CLAUDE.md"] {
            if tokio::fs::try_exists(workspace.join(name)).await? {
                let instruction = document(
                    &workspace,
                    Path::new(name),
                    format!("repository/existing/{name}"),
                )
                .await?;
                documents.insert(instruction.source.clone(), instruction);
            }
        }
    } else {
        let root = match root {
            Some(root) => relative(&root)?,
            None => steering(&workspace).await?.ok_or("Required root guidance is missing: select AGENTS.md or CLAUDE.md; Setup generation is a separate path")?,
        };
        let root_instruction = document(&workspace, &root, "repository/root".into()).await?;
        documents.insert(root_instruction.source.clone(), root_instruction);
    }
    for scope in &scopes {
        let scope = relative(scope)?;
        let directory = workspace.join(&scope);
        let canonical = tokio::fs::canonicalize(&directory)
            .await
            .context("Required instruction scope is unavailable")?;
        if !canonical.starts_with(&workspace) || !tokio::fs::metadata(&canonical).await?.is_dir() {
            return Err("Instruction scopes must be repository directories".into());
        }
        let mut current = PathBuf::new();
        for component in scope.components() {
            current.push(component);
            if let Some(path) = steering(&workspace.join(&current)).await? {
                let relative = current.join(path);
                let instruction = document(
                    &workspace,
                    &relative,
                    format!("repository/scope/{}", current.display()),
                )
                .await?;
                documents.insert(instruction.source.clone(), instruction);
            }
        }
    }
    for reference in &references {
        let path = relative(reference)?;
        let instruction = document(
            &workspace,
            &path,
            format!("repository/reference/{reference}"),
        )
        .await?;
        documents
            .entry(instruction.source.clone())
            .or_insert(instruction);
    }
    tokio::task::spawn_blocking(move || {
        let mut snapshot = domain::InstructionSnapshot {
            id: String::new(),
            phase,
            scopes,
            references,
            instructions: loaded
                .into_values()
                .chain(documents.into_values())
                .collect(),
        };
        snapshot.id = instructions::fingerprint(&snapshot);
        instructions::validate(&snapshot)?;
        Ok(snapshot)
    })
    .await?
}

fn relative(value: &str) -> Result<PathBuf, Error> {
    let path = Path::new(value);
    if value.is_empty()
        || value.len() > 4096
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(
            "Instruction paths must be nonempty repository-relative paths without traversal".into(),
        );
    }
    Ok(path.to_owned())
}

async fn steering(directory: &Path) -> Result<Option<PathBuf>, Error> {
    let mut found = Vec::new();
    for name in ["AGENTS.md", "CLAUDE.md"] {
        match tokio::fs::symlink_metadata(directory.join(name)).await {
            Ok(_) => found.push(PathBuf::from(name)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .context("Could not inspect repository guidance")
                    .map_err(Into::into);
            }
        }
    }
    if found.len() > 1 {
        return Err(format!("Ambiguous required root/scoped guidance in {}; select a root explicitly or resolve scoped ambiguity", directory.display()).into());
    }
    Ok(found.pop())
}

async fn document(workspace: &Path, path: &Path, id: String) -> Result<domain::Instruction, Error> {
    let canonical = tokio::fs::canonicalize(workspace.join(path))
        .await
        .map_err(|error| {
            Failure::caused_by(
                format!(
                    "Required repository instruction is unavailable: {}",
                    path.display()
                ),
                error,
            )
        })?;
    if !canonical.starts_with(workspace) {
        return Err("Required repository instruction resolves outside the workspace".into());
    }
    let bytes = read_required(&canonical, 262_144).await.map_err(|error| {
        Failure::caused_by(
            format!(
                "Could not read required repository instruction: {}",
                path.display()
            ),
            error,
        )
    })?;
    let body = String::from_utf8(bytes).context("Required repository instruction is not UTF-8")?;
    Ok(domain::Instruction {
        id,
        version: "captured-content".into(),
        source: canonical.to_string_lossy().into(),
        upstream: Vec::new(),
        requires: Vec::new(),
        body,
    })
}

async fn read_optional(path: &Path, limit: usize) -> Result<Option<Vec<u8>>, Error> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) if !metadata.is_file() => {
            return Err("Instructions must be regular files".into());
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !file.metadata().await?.is_file() {
        return Err("Instructions must be regular files".into());
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes).await?;
    if bytes.len() > limit {
        return Err("Required instruction file exceeds its size bound".into());
    }
    Ok(Some(bytes))
}

async fn read_required(path: &Path, limit: usize) -> Result<Vec<u8>, Error> {
    read_optional(path, limit)
        .await?
        .ok_or_else(|| "Required instruction file is missing".into())
}

async fn parse<T: serde::de::DeserializeOwned + Send + 'static>(
    bytes: Vec<u8>,
) -> Result<T, Error> {
    tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes).map_err(Into::into)).await?
}
