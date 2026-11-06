# Harness

Harness conversations contain user intent, model responses, and the outcomes of
requested actions. These terms distinguish a complete response from its individual
steps and preserve the meaning of interrupted work.

## Language

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
