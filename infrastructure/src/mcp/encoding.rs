use crate::tool_io::Error;
use application::error_context::ResultExt;
use std::io::Write;

struct Bounded {
    bytes: Vec<u8>,
    limit: usize,
}

impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("MCP JSON encoding exceeds its bound"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn bytes(value: &serde_json::Value, limit: usize) -> Result<Vec<u8>, Error> {
    let mut output = Bounded {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut output, value)
        .context("MCP JSON encoding failed or exceeded its bound")?;
    Ok(output.bytes)
}

pub(super) fn text(value: &serde_json::Value, limit: usize) -> Result<String, Error> {
    Ok(String::from_utf8(bytes(value, limit)?)?)
}
