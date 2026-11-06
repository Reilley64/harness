use crate::cache::json_string_len;
use crate::conversations;
use crate::storage;
use crate::tool_io::Error;
use std::path::Path;

pub(crate) const RESULT_TEXT_JSON_LIMIT: usize = 32768;

fn result_text_size(call: &domain::ToolCall) -> usize {
    if call
        .output
        .len()
        .saturating_add(call.artifact.as_ref().map_or(0, String::len))
        > RESULT_TEXT_JSON_LIMIT
    {
        return RESULT_TEXT_JSON_LIMIT + 1;
    }
    json_string_len(&call.output)
        .saturating_add(call.artifact.as_deref().map_or(4, json_string_len))
}

fn prefix_end(text: &str, budget: usize) -> usize {
    let mut size = 0;
    for (index, character) in text.char_indices() {
        let next = json_string_len(&text[index..index + character.len_utf8()]) - 2;
        if next > budget.saturating_sub(size) {
            return index;
        }
        size += next;
    }
    text.len()
}

fn truncate(
    mut call: domain::ToolCall,
    stored: bool,
    artifact: String,
) -> Result<domain::ToolCall, Error> {
    call.artifact = stored.then_some(artifact);
    let note = if stored {
        "\n[… output truncated; read the artifact for up to 1 MiB …]\n"
    } else {
        "\n[… output truncated; conversation artifact budget reached; no artifact was written …]\n"
    };
    let overhead =
        json_string_len(note).saturating_add(call.artifact.as_deref().map_or(4, json_string_len));
    let budget = RESULT_TEXT_JSON_LIMIT
        .checked_sub(overhead)
        .ok_or("Tool artifact reference exceeds its metadata limit")?;
    let head = prefix_end(&call.output, budget * 3 / 5);
    let mut tail = call.output.len();
    let mut tail_size = 0;
    for (index, character) in call.output.char_indices().rev() {
        let next = json_string_len(&call.output[index..index + character.len_utf8()]) - 2;
        if index < head || next > (budget - budget * 3 / 5).saturating_sub(tail_size) {
            break;
        }
        tail = index;
        tail_size += next;
    }
    call.output = format!("{}{note}{}", &call.output[..head], &call.output[tail..]);
    call.truncated = true;
    if result_text_size(&call) > RESULT_TEXT_JSON_LIMIT {
        return Err("Tool result exceeds its metadata limit".into());
    }
    Ok(call)
}

pub fn create_tool_output(
    root: &Path,
) -> impl Fn(String, domain::ToolCall) -> application::ports::PortFuture<Result<domain::ToolCall, Error>>
+ Send
+ Sync
+ use<> {
    let root = root.to_owned();
    move |id, call| {
        let root = root.clone();
        Box::pin(async move {
            conversations::validate_id(&id)?;
            conversations::validate_id(&call.id)?;
            let (call, oversized) = tokio::task::spawn_blocking(move || {
                let oversized = result_text_size(&call) > RESULT_TEXT_JSON_LIMIT;
                (call, oversized)
            })
            .await?;
            if !oversized {
                return Ok(call);
            }
            let directory = root.join("tool-outputs").join(id);
            storage::private_directory(&directory).await?;
            let path = directory.join(format!("{}.txt", call.id));
            let mut end = call.output.len().min(1_048_576);
            while !call.output.is_char_boundary(end) {
                end -= 1;
            }
            let mut size = 0u64;
            let mut entries = tokio::fs::read_dir(&directory).await?;
            while let Some(entry) = entries.next_entry().await? {
                let metadata = tokio::fs::symlink_metadata(entry.path()).await?;
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err("Invalid tool artifact entry".into());
                }
                size = size.saturating_add(metadata.len());
                if size > 128 * 1024 * 1024 {
                    break;
                }
            }
            let stored = size.saturating_add(end as u64) <= 128 * 1024 * 1024;
            if stored {
                storage::replace(&path, &call.output.as_bytes()[..end]).await?;
            }
            let artifact = path.to_string_lossy().into_owned();
            tokio::task::spawn_blocking(move || truncate(call, stored, artifact)).await?
        })
    }
}
