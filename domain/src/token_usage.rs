#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl TokenUsage {
    pub fn new(input_tokens: u64, output_tokens: u64) -> Result<Self, &'static str> {
        if input_tokens > 1_000_000_000 || output_tokens > 1_000_000_000 {
            return Err("Provider token usage exceeds its bound");
        }
        Ok(Self {
            input_tokens,
            output_tokens,
        })
    }
}
