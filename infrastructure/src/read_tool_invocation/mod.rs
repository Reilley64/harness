use crate::create_file;
use crate::create_mcp_call;
use crate::create_process;
use crate::read_code_matches;
use crate::read_directory;
use crate::read_file;
use crate::read_paths;
use crate::read_text_matches;
use crate::read_web_search;
use crate::read_work_proposal;
use crate::tool_io;
use crate::update_decision;
use crate::update_file;
use crate::update_mcp_configuration;

pub fn read_tool_invocation()
-> impl Fn(domain::Tool, String) -> Result<domain::ToolInvocation, tool_io::Error> + Send + Sync {
    |tool, arguments| {
        if arguments.len() > 65536 {
            return Err("Tool arguments exceed 64 KiB".into());
        }
        match tool {
            domain::Tool::McpCall => create_mcp_call::decode(&arguments),
            domain::Tool::McpConfiguration => update_mcp_configuration::decode(&arguments),
            domain::Tool::Read => read_file::decode(&arguments),
            domain::Tool::Bash => create_process::decode(&arguments),
            domain::Tool::Edit => update_file::decode(&arguments),
            domain::Tool::Write => create_file::decode(&arguments),
            domain::Tool::Ls => read_directory::decode(&arguments),
            domain::Tool::Find => read_paths::decode(&arguments),
            domain::Tool::Grep => read_text_matches::decode(&arguments),
            domain::Tool::AstSearch => read_code_matches::decode(&arguments),
            domain::Tool::WebSearch => read_web_search::decode(&arguments),
            domain::Tool::PublishWork => read_work_proposal::decode(&arguments),
            domain::Tool::PrepareAssignment => update_decision::decode(&arguments),
        }
    }
}
