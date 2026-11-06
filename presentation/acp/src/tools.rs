pub(crate) fn summary(message_id: &str, call: &domain::ToolCall) -> serde_json::Value {
    serde_json::json!({"messageId":message_id,"id":call.id,"name":call.name,"arguments":call.arguments,"status":call.status.as_str(),"output":call.output,"artifact":call.artifact,"truncated":call.truncated,"elapsedMs":call.elapsed_ms,"textOffset":call.text_offset})
}
