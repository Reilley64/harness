use crate::tool_io::Error;
use application::error_context::ResultExt;
use serde_json::Value;
use std::io::Write;

const MARKER: &str = "[redacted]";

pub(super) struct Budget {
    bytes: usize,
    nodes: usize,
    comparisons: usize,
}

impl Budget {
    pub(super) fn new(bytes: usize) -> Self {
        Self {
            bytes,
            nodes: 16_384,
            comparisons: 67_108_864,
        }
    }

    fn spend(&mut self, bytes: usize) -> Result<(), Error> {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or("MCP public projection exceeds its byte budget")?;
        Ok(())
    }

    fn node(&mut self) -> Result<(), Error> {
        self.nodes = self
            .nodes
            .checked_sub(1)
            .ok_or("MCP public projection exceeds its container budget")?;
        Ok(())
    }
}

pub(super) fn encoded_size(text: &str) -> usize {
    2 + text.chars().map(escaped_size).sum::<usize>()
}

fn escaped_size(character: char) -> usize {
    match character {
        '"' | '\\' | '\u{08}' | '\t' | '\n' | '\u{0c}' | '\r' => 2,
        '\u{00}'..='\u{1f}' => 6,
        _ => character.len_utf8(),
    }
}

pub(super) fn text(
    original: &str,
    secrets: &[String],
    budget: &mut Budget,
) -> Result<String, Error> {
    budget.node()?;
    budget.spend(2)?;
    let mut output = String::new();
    let mut remaining = original;
    while !remaining.is_empty() {
        let mut matched = None;
        for secret in secrets {
            budget.comparisons = budget
                .comparisons
                .checked_sub(secret.len().min(remaining.len()))
                .ok_or("MCP public projection exceeds its comparison budget")?;
            if remaining.starts_with(secret) {
                matched = Some(secret.len());
                break;
            }
        }

        if let Some(length) = matched {
            budget.spend(MARKER.len())?;
            output.push_str(MARKER);
            remaining = &remaining[length..];
        } else {
            let character = remaining.chars().next().ok_or("Missing MCP text")?;
            budget.spend(escaped_size(character))?;
            output.push(character);
            remaining = &remaining[character.len_utf8()..];
        }
    }
    Ok(output)
}

fn project_value(value: &mut Value, secrets: &[String], budget: &mut Budget) -> Result<(), Error> {
    budget.node()?;
    match value {
        Value::String(original) => *original = text(original, secrets, budget)?,
        Value::Array(values) => {
            budget.spend(2 + values.len().saturating_sub(1))?;
            for entry in values {
                project_value(entry, secrets, budget)?;
            }
        }
        Value::Object(values) => {
            budget.spend(2 + values.len() + values.len().saturating_sub(1))?;
            let original = std::mem::take(values);
            for (key, mut entry) in original {
                let key = text(&key, secrets, budget)?;
                if values.contains_key(&key) {
                    return Err("MCP public projection has colliding object keys".into());
                }
                project_value(&mut entry, secrets, budget)?;
                values.insert(key, entry);
            }
        }
        Value::Null => budget.spend(4)?,
        Value::Bool(boolean) => budget.spend(if *boolean { 4 } else { 5 })?,
        Value::Number(_) => budget.spend(32)?,
    }
    Ok(())
}

struct Encoding {
    bytes: Vec<u8>,
    limit: usize,
    outer_remaining: usize,
}

impl Write for Encoding {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let text = std::str::from_utf8(bytes).map_err(std::io::Error::other)?;
        let outer = encoded_size(text) - 2;
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) || outer > self.outer_remaining
        {
            return Err(std::io::Error::other(
                "MCP public encoding exceeds its byte budget",
            ));
        }
        self.outer_remaining -= outer;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn json(
    mut original: Value,
    secrets: &[String],
    limit: usize,
    outer_limit: usize,
) -> Result<String, Error> {
    project_value(&mut original, secrets, &mut Budget::new(limit))?;
    let mut encoding = Encoding {
        bytes: Vec::new(),
        limit,
        outer_remaining: outer_limit
            .checked_sub(2)
            .ok_or("MCP public encoding exceeds its byte budget")?,
    };
    serde_json::to_writer(&mut encoding, &original)
        .context("MCP public encoding exceeds its byte budget")?;
    Ok(String::from_utf8(encoding.bytes)?)
}
