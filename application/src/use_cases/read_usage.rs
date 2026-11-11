mod dependencies;
mod error;
mod output;
mod projection;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    conversation_id: Option<String>,
    run_id: Option<String>,
    work_id: Option<String>,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    if conversation_id.is_some() == work_id.is_some()
        || run_id.is_some() && conversation_id.is_none()
    {
        return Err(Error::blocked(
            "Choose one conversation or approved-work usage scope; a run requires its conversation",
        ));
    }
    for id in [&conversation_id, &run_id, &work_id].into_iter().flatten() {
        if !uuid::Uuid::parse_str(id).is_ok_and(|uuid| uuid.to_string() == *id) {
            return Err(Error::blocked("Usage identifiers must be canonical UUIDs"));
        }
    }

    let dependencies = dependencies.into();
    let sources = dependencies
        .read_usage_sources
        .call(cancellation.clone())
        .await
        .map_err(Error::unavailable)?;

    let mut native = Vec::new();
    let mut needed_runs = BTreeMap::<String, HashSet<String>>::new();
    let mut retained = 0usize;
    let mut known_work = work_id.is_none();
    let mut parent_sources = BTreeSet::new();

    for id in sources.decisions {
        if cancellation.is_cancelled() {
            return Err(Error::caused_by("Usage query cancelled"));
        }
        let decision = dependencies
            .read_decision
            .call(id)
            .await
            .map_err(Error::caused_by)?
            .ok_or_else(|| Error::blocked("Usage decision disappeared; refresh accounting"))?;
        if conversation_id
            .as_ref()
            .is_some_and(|id| id != &decision.parent_id)
            || work_id.as_ref().is_some_and(|id| id != &decision.work_id)
            || run_id
                .as_ref()
                .is_some_and(|id| decision.parent_run_id.as_ref().is_some_and(|run| run != id))
        {
            continue;
        }
        known_work = true;
        parent_sources.insert(decision.parent_id.clone());
        let wanted_run = run_id.clone();
        if let Some(source) = tokio::task::spawn_blocking(move || {
            let mut source = projection::native(decision)?;
            if wanted_run.is_some() && source.run.is_none() {
                source.entries = Vec::new();
            }
            Some(source)
        })
        .await
        .map_err(Error::caused_by)?
        {
            retained = retained.saturating_add(source.entries.len());
            capacity(retained)?;
            if let Some(run) = &source.run {
                needed_runs
                    .entry(source.parent.clone())
                    .or_default()
                    .insert(run.clone());
            }
            native.push(source);
        }
    }
    if let (Some(id), Some(run)) = (&conversation_id, &run_id) {
        needed_runs
            .entry(id.clone())
            .or_default()
            .insert(run.clone());
    }

    let mut ownership = BTreeMap::new();
    let mut pending = conversation_id
        .as_ref()
        .map_or_else(|| sources.conversations.clone(), |id| vec![id.clone()]);

    while let Some(id) = pending.pop() {
        if cancellation.is_cancelled() {
            return Err(Error::caused_by("Usage query cancelled"));
        }
        if ownership.contains_key(&id) || !sources.conversations.contains(&id) {
            return Err(Error::blocked("Usage ownership is cyclic or unavailable"));
        }
        let conversation = dependencies
            .read_conversation
            .call(id.clone())
            .await
            .map_err(Error::caused_by)?;

        let runs = needed_runs.remove(&id).unwrap_or_default();
        let header =
            tokio::task::spawn_blocking(move || projection::ownership(&conversation, &runs))
                .await
                .map_err(Error::caused_by)?;

        if conversation_id.is_some()
            && let Some(parent) = &header.parent
        {
            if ownership.len() >= 256 {
                return Err(Error::blocked("Usage ownership exceeds its bound"));
            }
            pending.push(parent.clone());
        }
        ownership.insert(id, header);
    }

    if let (Some(id), Some(run)) = (&conversation_id, &run_id)
        && ownership
            .get(id)
            .is_none_or(|source| !source.known_runs.contains(run))
    {
        return Err(Error::blocked("Usage run is unavailable"));
    }
    for source in &native {
        if let Some(run) = &source.run
            && ownership
                .get(&source.parent)
                .is_none_or(|parent| !parent.known_runs.contains(run))
        {
            return Err(Error::blocked(
                "Native accounting run provenance is unavailable",
            ));
        }
    }

    if let Some(work_id) = &work_id {
        known_work |= ownership
            .values()
            .any(|source| source.work.as_ref() == Some(work_id));
    }
    if !known_work {
        return Err(Error::blocked(
            "Usage work scope is unavailable; inspect its retained assignment or intent before querying",
        ));
    }

    let mut entries = Vec::new();
    let mut unknown_history_sources: Vec<_> = parent_sources
        .into_iter()
        .filter(|id| !ownership.contains_key(id))
        .map(|id| format!("{id}:unavailable"))
        .collect();
    retained = retained.saturating_add(unknown_history_sources.len());
    capacity(retained)?;

    for (id, header) in &ownership {
        if conversation_id
            .as_ref()
            .is_some_and(|selected| selected != id)
        {
            continue;
        }
        let owner_work = projection::work(&ownership, id)?;
        if work_id
            .as_ref()
            .is_some_and(|id| Some(id) != owner_work.as_ref())
        {
            continue;
        }
        if cancellation.is_cancelled() {
            return Err(Error::caused_by("Usage query cancelled"));
        }
        let conversation = dependencies
            .read_conversation
            .call(id.clone())
            .await
            .map_err(Error::caused_by)?;

        let wanted_run = run_id.clone();
        let known_runs = header.known_runs.clone();

        let mut source = tokio::task::spawn_blocking(move || {
            projection::conversation(conversation, wanted_run, known_runs)
        })
        .await
        .map_err(Error::caused_by)??;

        if source.id != *id
            || source.ownership.parent != header.parent
            || source.ownership.work != header.work
            || source.ownership.known_runs != header.known_runs
        {
            return Err(Error::blocked(
                "Usage ownership changed during the query; refresh accounting",
            ));
        }

        retained = retained
            .saturating_add(source.entries.len())
            .saturating_add(source.unknown_history.len());
        capacity(retained)?;
        for entry in &mut source.entries {
            entry.work_id = owner_work.clone();
        }
        entries.extend(source.entries);
        unknown_history_sources.extend(source.unknown_history);
    }

    tokio::task::spawn_blocking(move || {
        let mut unassigned_native_runs = 0;
        for source in native {
            unassigned_native_runs += source.unassigned;
            if run_id
                .as_ref()
                .is_some_and(|run| Some(run) != source.run.as_ref())
            {
                continue;
            }
            entries.extend(source.entries);
        }

        let mut unique = BTreeMap::new();
        for entry in entries {
            if unique.insert(entry.id.clone(), entry).is_some() {
                return Err(Error::blocked(
                    "Conflicting usage attempt or stage identity",
                ));
            }
        }

        Ok(Output {
            entries: unique.into_values().collect(),
            unassigned_native_runs,
            unknown_history_sources,
        })
    })
    .await
    .map_err(Error::caused_by)?
}

fn capacity(entries: usize) -> Result<(), Error> {
    if entries > 100_000 {
        return Err(Error::blocked(
            "Usage entry bound reached; required facts cannot be omitted",
        ));
    }
    Ok(())
}
