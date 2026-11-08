use crate::ports::UpdateMcpConfiguration;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) update_mcp_configuration: UpdateMcpConfiguration,
}
