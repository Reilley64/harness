use crate::AssignmentBrief;
use crate::CreateFileArguments;
use crate::CreateProcessArguments;
use crate::McpCallArguments;
use crate::McpConfigurationAction;
use crate::PublishWorkArguments;
use crate::ReadCodeMatchesArguments;
use crate::ReadDirectoryArguments;
use crate::ReadFileArguments;
use crate::ReadPathsArguments;
use crate::ReadTextMatchesArguments;
use crate::ReadWebSearchArguments;
use crate::UpdateFileArguments;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolInvocation {
    McpCall(McpCallArguments),
    McpConfiguration(McpConfigurationAction),
    ReadFile(ReadFileArguments),
    CreateProcess(CreateProcessArguments),
    UpdateFile(UpdateFileArguments),
    CreateFile(CreateFileArguments),
    ReadDirectory(ReadDirectoryArguments),
    ReadPaths(ReadPathsArguments),
    ReadTextMatches(ReadTextMatchesArguments),
    ReadCodeMatches(ReadCodeMatchesArguments),
    ReadWebSearch(ReadWebSearchArguments),
    PublishWork(PublishWorkArguments),
    PrepareAssignment(AssignmentBrief),
}
