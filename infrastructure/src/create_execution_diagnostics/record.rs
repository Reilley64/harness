use super::Error;
use crate::conversations;
use std::fmt::Write;

const LIMIT: usize = 4 * 1024 * 1024;

#[derive(serde::Serialize)]
struct Record {
    format: u32,
    parent_id: String,
    execution_id: String,

    diagnostics: Vec<Diagnostic>,
}

#[derive(serde::Serialize)]
struct Diagnostic {
    debug: String,
    causes: Vec<String>,
}

struct Text {
    remaining: usize,
    value: String,
}

impl Write for Text {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        if text.len() > self.remaining {
            return Err(std::fmt::Error);
        }
        self.remaining -= text.len();
        self.value.push_str(text);
        Ok(())
    }
}

struct Encoded(Vec<u8>);

impl std::io::Write for Encoded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > LIMIT.saturating_sub(self.0.len()) {
            return Err(std::io::Error::other(
                "Diagnostic encoding exceeds its private bound",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn text(arguments: std::fmt::Arguments<'_>, remaining: &mut usize) -> Result<String, Error> {
    let mut buffer = Text {
        remaining: *remaining,
        value: String::new(),
    };
    buffer.write_fmt(arguments).map_err(
        |_| "Diagnostic text exceeds its private bound; originals retained by the runtime error",
    )?;
    *remaining = buffer.remaining;
    Ok(buffer.value)
}

pub(super) fn encode(
    parent_id: String,
    execution_id: String,
    diagnostics: &[Error],
) -> Result<Vec<u8>, Error> {
    conversations::validate_id(&parent_id)?;
    conversations::validate_id(&execution_id)?;
    let mut entries = Vec::new();
    let mut remaining = LIMIT;
    for error in diagnostics {
        let debug = text(format_args!("{error:?}"), &mut remaining)?;
        let mut causes = Vec::new();
        let mut source = error.source();
        while let Some(cause) = source {
            if causes.len() >= 256 {
                return Err("Diagnostic chain exceeds the private record bound; originals retained by the runtime error".into());
            }
            causes.push(text(format_args!("{cause}"), &mut remaining)?);
            source = cause.source();
        }
        entries.push(Diagnostic { debug, causes });
    }
    let mut encoded = Encoded(Vec::new());
    serde_json::to_writer(
        &mut encoded,
        &Record {
            format: 1,
            parent_id,
            execution_id,
            diagnostics: entries,
        },
    )?;
    Ok(encoded.0)
}
