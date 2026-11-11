use crate::CapturedJoin;
use crate::ChildExecution;
use crate::ChildOutcome;
use crate::CoordinationGate;
use crate::ScopeAmendment;

#[derive(Debug, Clone, PartialEq)]
pub enum CoordinationCommand {
    SteerChild {
        id: String,
        execution_id: String,
        text: String,
    },
    RecoverChild {
        execution_id: String,
        notice_id: String,
    },
    RetainRecovery {
        agent_id: String,
        notices: Vec<String>,
        idle: bool,
    },
    ResumeIdle {
        agent_id: String,
    },
    RequestScope(ScopeAmendment),
    ApproveScope {
        id: String,
        version: String,
        approval_id: String,
    },
    ResumeChild {
        execution_id: String,
    },
    AnswerChild {
        execution_id: String,
        request_run: String,
        approval_id: Option<String>,
    },
    AdmitDescendantAction {
        execution_id: String,
        controller: String,
    },
    AdmitChildAction {
        execution_id: String,
        controller: String,
    },
    ApplyChildInstruction {
        id: String,
        controller: String,
        run_id: String,
        reply_id: String,
    },
    Dispatch(Box<ChildExecution>),
    RetainClaim(Box<ChildExecution>),
    Start {
        execution_id: String,
        controller: String,
    },
    MarkUnknown {
        execution_id: String,
        controller: String,
    },
    StopChild {
        execution_id: String,
    },
    Stop {
        agent_id: String,
    },
    Receive {
        execution_id: String,
        controller: String,
        outcome: ChildOutcome,
    },
    Join(CapturedJoin),
    Background {
        join_id: String,
        run_id: String,
    },
    Acquire {
        agent_id: String,

        controller: String,
        run_id: String,
    },
    ResumeInputParent {
        agent_id: String,
        controller: String,
        run_id: String,
    },
    ResumeParent {
        agent_id: String,
        controller: String,
        run_id: String,
    },
    AcknowledgeParent {
        agent_id: String,
        controller: String,
        run_id: String,
    },
    Wait {
        controller: String,
        run_id: String,
    },
    Deliver {
        controller: String,
        run_id: String,
        outcomes: Vec<String>,
    },
    CompleteFollowUp {
        controller: String,
        run_id: String,
        outcomes: Vec<String>,
    },
    Release {
        controller: String,
        run_id: String,
    },
    Gate {
        gate: CoordinationGate,

        controller: Option<String>,
        run_id: Option<String>,
    },
}
