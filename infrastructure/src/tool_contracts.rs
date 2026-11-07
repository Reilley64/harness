use crate::create_file;
use crate::create_process;
use crate::read_code_matches;
use crate::read_directory;
use crate::read_file;
use crate::read_paths;
use crate::read_text_matches;
use crate::read_web_search;
use crate::read_work_proposal;
use crate::update_file;

pub(crate) fn definitions(tools: &[domain::Tool]) -> serde_json::Value {
    serde_json::Value::Array(
        tools
            .iter()
            .map(|tool| match tool {
                domain::Tool::Read => read_file::definition(),
                domain::Tool::Bash => create_process::definition(),
                domain::Tool::Edit => update_file::definition(),
                domain::Tool::Write => create_file::definition(),
                domain::Tool::Ls => read_directory::definition(),
                domain::Tool::Find => read_paths::definition(),
                domain::Tool::Grep => read_text_matches::definition(),
                domain::Tool::AstSearch => read_code_matches::definition(),
                domain::Tool::WebSearch => read_web_search::definition(),
                domain::Tool::PublishWork => read_work_proposal::definition(),
            })
            .collect(),
    )
}
