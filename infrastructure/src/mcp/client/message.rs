use crate::tool_io::Error;
use application::error_context::ResultExt;
use serde_json::Value;

pub(super) enum Message {
    Request(Value),
    Notification,
    Result(Value),
}

pub(super) struct RequestRejected {
    code: i64,
    frame: Vec<u8>,
}

impl std::fmt::Debug for RequestRejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RequestRejected")
            .field("code", &self.code)
            .field("frame_bytes", &self.frame.len())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for RequestRejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .write_str("MCP server rejected the request; verify access and operation availability")
    }
}

impl std::error::Error for RequestRejected {}

pub(super) fn parse(bytes: Vec<u8>, expected: u64) -> Result<Message, Error> {
    let mut response: Value =
        serde_json::from_slice(&bytes).context("Invalid MCP JSON-RPC response")?;
    if !response.is_object() || response["jsonrpc"] != "2.0" {
        return Err("Invalid MCP JSON-RPC version or envelope".into());
    }
    if let Some(method) = response.get("method") {
        if method
            .as_str()
            .is_none_or(|method| method.is_empty() || method.len() > 256)
            || response
                .get("params")
                .is_some_and(|params| !params.is_object())
            || response.get("result").is_some()
            || response.get("error").is_some()
        {
            return Err("Invalid MCP server request or notification".into());
        }
        return match response.get_mut("id") {
            Some(id)
                if id
                    .as_str()
                    .is_some_and(|id| !id.is_empty() && id.len() <= 128)
                    || id.as_i64().is_some()
                    || id.as_u64().is_some() =>
            {
                Ok(Message::Request(id.take()))
            }
            Some(_) => Err("Invalid MCP server request identity".into()),
            None => Ok(Message::Notification),
        };
    }
    if response["id"] != expected {
        return Err("MCP response identity mismatch".into());
    }
    if response.get("error").is_some() == response.get("result").is_some() {
        return Err("MCP response must contain exactly one result or error".into());
    }
    if let Some(error) = response.get_mut("error") {
        if !error.is_object() || error["code"].as_i64().is_none() || !error["message"].is_string() {
            return Err("Malformed MCP error response".into());
        }
        return Err(RequestRejected {
            code: error["code"].as_i64().ok_or("Invalid MCP rejection code")?,
            frame: bytes,
        }
        .into());
    }
    Ok(Message::Result(
        response
            .get_mut("result")
            .map(Value::take)
            .ok_or("Missing MCP result")?,
    ))
}
