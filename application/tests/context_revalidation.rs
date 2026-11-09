use application::ports::ReadCoordination;
use application::ports::ReadWork;
use domain::ApprovedWork;
use domain::AssignmentBrief;
use domain::ChildExecution;
use domain::ChildExecutionStatus;
use domain::Coordination;
use domain::CoordinationGate;
use domain::DecisionCandidate;
use domain::Instruction;
use domain::InstructionSnapshot;
use domain::Model;
use domain::ModelCapability;
use domain::ModelSelection;
use domain::ProviderId;
use domain::ResearchAssignment;
use domain::WorkApproval;
use domain::WorkProposal;
use domain::WorkPublication;

fn model(context_window: u64) -> Model {
    Model {
        id: "controlled-worker".into(),
        canonical_id: Some("controlled-worker-canonical".into()),
        name: "Controlled worker".into(),
        context_window: Some(context_window),
        capabilities: vec![ModelCapability::Chat],
        provider_id: ProviderId("openrouter".into()),
        efforts: Vec::new(),
        pricing: None,
    }
}

fn instructions() -> InstructionSnapshot {
    InstructionSnapshot {
        id: "a".repeat(64),
        phase: "research".into(),
        scopes: Vec::new(),
        references: Vec::new(),
        instructions: ["role/research", "contract", "procedure/research"]
            .into_iter()
            .map(|id| Instruction {
                id: id.into(),
                version: "controlled".into(),
                source: "controlled-test".into(),
                upstream: Vec::new(),
                requires: Vec::new(),
                body: "Controlled temporary external integration-test instruction.".into(),
            })
            .collect(),
    }
}

fn execution(routed: bool) -> ChildExecution {
    let selection = ModelSelection {
        provider_id: ProviderId("openrouter".into()),
        model_id: "controlled-worker".into(),
        effort: None,
    };
    ChildExecution {
        intent: "intent".into(),
        assignment_id: "assignment".into(),
        agent_id: "research-agent".into(),
        execution_id: "execution".into(),

        parent_id: "parent-conversation".into(),
        parent_agent_id: "parent-agent".into(),
        parent_run_id: "parent-run".into(),
        parent_revision: 1,

        parent_selection: selection.clone(),
        parent_connection_id: "connection".into(),

        work_id: routed.then(|| "work".into()),
        work_intent: routed.then(|| "approval-intent".into()),
        work_version: routed.then(|| "b".repeat(64)),
        approved_at: routed.then_some(42),

        brief: AssignmentBrief {
            intent: "intent".into(),
            role: "research".into(),
            task: "Controlled context admission".into(),
            constraints: String::new(),
            tool_context: String::new(),
            context_tokens: 4096,
        },
        selected: DecisionCandidate {
            key: "candidate".into(),
            selection,
            connection_id: "connection".into(),
            model: model(8192),
        },
        instructions: instructions(),
        routed,

        controller: "controller".into(),
        status: ChildExecutionStatus::Running,
        stop_requested: false,

        outcome: None,

        delivered_to: None,
        follow_up_completed_by: None,
    }
}

fn approved_work() -> ApprovedWork {
    let proposal = WorkProposal {
        root: "file:proposal".into(),
        source_revision: "controlled".into(),
        spec: "Controlled specification".into(),
        scope: "Controlled scope".into(),
        criteria: "Controlled criteria".into(),
        dependencies: "No dependencies".into(),
        wiki: None,
        context: None,
        artifacts: Vec::new(),
        document: "Controlled document".into(),
        version: "b".repeat(64),
    };
    ApprovedWork {
        id: "work".into(),
        creator_id: "creator".into(),
        conversation_id: "griller".into(),
        workspace: std::path::PathBuf::from("/controlled"),
        revision: 1,
        publications: vec![WorkPublication {
            intent: "publication".into(),
            proposal,
        }],
        approvals: vec![WorkApproval {
            intent: "approval-intent".into(),
            version: "b".repeat(64),
            revision: 1,
            selection: ModelSelection {
                provider_id: ProviderId("openrouter".into()),
                model_id: "controlled-worker".into(),
                effort: None,
            },
            coordinator_id: "parent-conversation".into(),
            approved_at: 42,
            delivered: true,
        }],
    }
}

fn dependencies(routed: bool) -> (ReadCoordination, ReadWork, ResearchAssignment) {
    let child = execution(routed);
    child.validate().unwrap();
    let research = ResearchAssignment {
        parent_id: child.parent_id.clone(),
        parent_agent_id: child.parent_agent_id.clone(),
        intent: child.intent.clone(),
        assignment_id: child.assignment_id.clone(),
        execution_id: child.execution_id.clone(),
    };
    let mut state = Coordination::new(child.parent_id.clone(), child.parent_agent_id.clone());
    state.gate = CoordinationGate::Open;
    state.controller = Some("parent-controller".into());
    state.run_id = Some(child.parent_run_id.clone());
    state.children.push(child);
    state.validate().unwrap();
    let coordination = ReadCoordination::new(move |_| {
        let state = state.clone();
        Box::pin(async move { Ok(Some(state)) })
    });
    let approved = approved_work();
    let work = ReadWork::new(move |_| {
        let approved = approved.clone();
        Box::pin(async move { Ok(routed.then_some(approved)) })
    });
    (coordination, work, research)
}

async fn admission(
    routed: bool,
    context_window: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (coordination, work, research) = dependencies(routed);
    application::research_access::read(
        &coordination,
        &work,
        Some(&research),
        "research-agent",
        &model(context_window),
        None,
        "connection",
        Some(&"a".repeat(64)),
    )
    .await
}

#[tokio::test]
async fn refreshed_context_capacity_fences_unrouted_research() {
    assert!(admission(false, 1024).await.is_err());
}

#[tokio::test]
async fn refreshed_context_capacity_fences_routed_research() {
    assert!(admission(true, 1024).await.is_err());
}

#[tokio::test]
async fn unchanged_sufficient_context_remains_admitted() {
    admission(false, 8192).await.unwrap();
    admission(true, 8192).await.unwrap();
}
