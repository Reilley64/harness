#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationCycles {
    pub allowance: u32,
    pub rework_runs: Vec<String>,
}

impl Default for ImplementationCycles {
    fn default() -> Self {
        Self {
            allowance: 3,
            rework_runs: Vec::new(),
        }
    }
}

impl ImplementationCycles {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.allowance != Self::default().allowance
            || self.rework_runs.len() >= self.allowance as usize
            || self.rework_runs.iter().enumerate().any(|(index, run)| {
                run.is_empty() || run.len() > 128 || self.rework_runs[..index].contains(run)
            })
        {
            return Err("Implementation cycle history differs from its immutable allowance");
        }
        Ok(())
    }

    pub fn admit(&mut self, source_run: String) -> Result<bool, &'static str> {
        self.validate()?;
        if self.rework_runs.contains(&source_run) {
            return Ok(false);
        }
        if source_run.is_empty() || source_run.len() > 128 {
            return Err("Implementation rework requires its exact source run");
        }
        if self.rework_runs.len() + 1 >= self.allowance as usize {
            return Err("The recorded implementation cycle allowance is exhausted");
        }

        self.rework_runs.push(source_run);
        Ok(true)
    }
}
