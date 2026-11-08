use crate::tool_io::Error;
use application::error_context::ResultExt;

const LIMIT: usize = 1_048_576;

pub(super) struct Decoder {
    buffer: Vec<u8>,
    data: Vec<u8>,
    offset: usize,
    scanned: usize,
    total: usize,
    lines: usize,
    events: usize,
    skip_lf: bool,
    first: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            data: Vec::new(),
            offset: 0,
            scanned: 0,
            total: 0,
            lines: 0,
            events: 0,
            skip_lf: false,
            first: true,
        }
    }
}

impl Decoder {
    pub(super) fn append(&mut self, chunk: &[u8]) -> Result<(), Error> {
        self.total = self
            .total
            .checked_add(chunk.len())
            .ok_or("MCP SSE byte count overflow")?;
        if self.total > LIMIT * 8 || self.buffer.len() - self.offset + chunk.len() > LIMIT {
            return Err("MCP SSE stream or frame exceeds its bound".into());
        }
        if self.offset != 0 {
            self.buffer.drain(..self.offset);
            self.scanned -= self.offset;
            self.offset = 0;
        }
        self.buffer.extend_from_slice(chunk);
        Ok(())
    }

    pub(super) fn next(&mut self) -> Result<Option<Vec<u8>>, Error> {
        loop {
            let Some(relative) = self.buffer[self.scanned..]
                .iter()
                .position(|byte| matches!(byte, b'\r' | b'\n'))
            else {
                self.scanned = self.buffer.len();
                return Ok(None);
            };
            let position = self.scanned + relative;
            let terminator = self.buffer[position];
            let mut line = &self.buffer[self.offset..position];
            self.offset = position + 1;
            self.scanned = self.offset;
            if self.skip_lf && terminator == b'\n' && line.is_empty() {
                self.skip_lf = false;
                continue;
            }
            self.skip_lf = terminator == b'\r';
            self.lines += 1;
            if self.lines > 4096 {
                return Err("MCP SSE line limit exceeded".into());
            }
            if self.first {
                self.first = false;
                if line.starts_with(&[0xef, 0xbb, 0xbf]) {
                    line = &line[3..];
                }
            }
            if line.is_empty() {
                if self.data.is_empty() {
                    continue;
                }
                self.events += 1;
                if self.events > 256 {
                    return Err("MCP SSE event limit exceeded".into());
                }
                self.data.pop();
                return Ok(Some(std::mem::take(&mut self.data)));
            }
            let line = std::str::from_utf8(line).context("MCP SSE is not valid UTF-8")?;
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.strip_prefix(' ').unwrap_or(data);
                if self.data.len() + data.len() + 1 > LIMIT {
                    return Err("MCP SSE event exceeds 1 MiB".into());
                }
                self.data.extend_from_slice(data.as_bytes());
                self.data.push(b'\n');
            } else if let Some(event) = line.strip_prefix("event:")
                && !matches!(event.trim(), "message" | "")
            {
                return Err(
                    "Unsupported MCP SSE event; legacy endpoint events are not followed".into(),
                );
            }
        }
    }
}
