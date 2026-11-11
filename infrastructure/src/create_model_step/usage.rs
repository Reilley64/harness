use super::ModelStepError;

pub(super) fn cost(event: &serde_json::Value) -> Result<Option<f64>, ModelStepError> {
    let Some(value) = event
        .pointer("/response/usage/cost")
        .filter(|value| !value.is_null())
    else {
        return Ok(None);
    };
    Ok(Some(
        value
            .as_f64()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .ok_or("Invalid reported provider cost")?,
    ))
}

pub(super) fn read(
    event: &serde_json::Value,
) -> Result<Option<domain::TokenUsage>, ModelStepError> {
    let Some(usage) = event
        .pointer("/response/usage")
        .filter(|value| !value.is_null())
    else {
        return Ok(None);
    };
    let input = usage["input_tokens"]
        .as_u64()
        .ok_or("Invalid provider input token usage")?;
    let output = usage["output_tokens"]
        .as_u64()
        .ok_or("Invalid provider output token usage")?;
    Ok(Some(domain::TokenUsage::new(input, output)?))
}
