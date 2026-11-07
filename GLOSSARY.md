# Harness

Harness conversations contain user intent, model responses, and the outcomes of
requested actions. These terms distinguish a complete response from its individual
steps and preserve the meaning of interrupted work.

## Language

**Agent**:
A durable, role-bound participant assigned to work. Its identity persists across
model invocations and changes in model selection; multiple agents may share a role.

**System 0 agent**:
A conversational agent that carries out tool-assisted work. The Decider, which
returns typed decisions rather than performing tool-assisted work, is not a
System 0 agent.

**Model allowlist**:
The user's set of model identities permitted for inference, shared by workers and
the Decider. Membership alone does not establish authentication or compatibility
with assigned work.

**Task**:
A kind of work an agent performs, defined once and executable many times, such as
research, implementation, or review.
_Avoid_: Job, workflow, action

**Run**:
One execution of a task by an agent, from its trigger to a terminal outcome.
Waiting for delegated work may suspend the same run; a human answer starts a later
run after the preceding run ends for input.

**Ticket acceptance**:
The state in which a ticket's submitted changes have met required review and
checks and have been integrated into its spec. It does not imply tracker closure
or final merge.

**Spec completion**:
The state in which a spec's accepted changes have been verified as merged into
their target branch. PR readiness and cleanup are separate states.

**Conversation**:
A durable history of interactions associated with a provider connection and a
selected model. It can be continued through multiple turns.

**Turn**:
One user message and the model steps and tool calls undertaken to answer it.
_Avoid_: Response, when referring to the entire interaction rather than one model step.

**Model step**:
One model generation that produces assistant content, requests tools, or ends the
turn. A completed model step does not necessarily complete the turn.

**Token usage**:
Provider-reported input and output token counts for a model generation. Reported
work counts even when its content is superseded or cancelled; an absent report
is unknown, not zero.

**Resource reference**:
A named reference to content at a location. A reference does not imply that its
content has been accessed.

**Tool call**:
An identified request for a particular tool to act on supplied arguments. A tool
call may be rejected or cancelled without execution.

**Tool execution**:
The actual action undertaken for a valid, enabled tool call. Stopping an execution
does not necessarily undo effects it has already produced.

**Tool result**:
The recorded outcome of a tool call, including success, failure, cancellation,
or an outcome that cannot be established.

**Steering instruction**:
A user instruction submitted while a turn is active that changes how the remaining
work should proceed without erasing actions already performed.
