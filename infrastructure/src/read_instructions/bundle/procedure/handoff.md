# Message-first handoff

Compose for the recipient's next assigned work, at actual fresh dispatch/return boundaries. Include assignment/spec/ticket/work revision; outcome (completed, blocked, needs input, failed or stopped); concise context; durable artifact pointers; verification/findings; known/uncertain effects; requested follow-up and relevant procedures.

Reference existing specs/issues/commits/reports instead of copying them. Redact secrets and personal data. Use the real durable message path: sender/recipient/work-version provenance must be recorded before delivery/continuation. If messaging is unavailable, return the report to the caller without pretending delivery occurred.

Hard Stop/crash retains actual records; it must not start inference to fabricate a final handoff. A message or summary cannot grant scope, settle work, reset allowances, authorize writes/merge or resolve Unknown outcomes.
