use application::error_context::ResultExt;

use super::ModelStepError;

#[derive(Default)]
pub(super) struct Decoder {
    line: Vec<u8>,
    data: Vec<String>,
    size: usize,
}

impl Decoder {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<serde_json::Value>, ModelStepError> {
        let mut events = Vec::new();
        for byte in bytes {
            if *byte != b'\n' {
                self.line.push(*byte);
                if self.line.len().saturating_add(self.size) > 4_194_304 {
                    return Err("Provider stream event exceeds the size limit".into());
                }
                continue;
            }
            let line = String::from_utf8(std::mem::take(&mut self.line))
                .context("Invalid UTF-8 in provider stream")?;
            let line = line.trim_end_matches('\r');
            if line.is_empty() {
                if !self.data.is_empty() {
                    let data = self.data.join("\n");
                    self.data.clear();
                    self.size = 0;
                    if data != "[DONE]" {
                        events.push(
                            serde_json::from_str(&data)
                                .context("Invalid JSON in provider stream")?,
                        );
                    }
                }
            } else if let Some(data) = line.strip_prefix("data:") {
                let data = data.strip_prefix(' ').unwrap_or(data);
                self.size += data.len();
                self.data.push(data.to_owned());
            }
        }
        Ok(events)
    }
}
