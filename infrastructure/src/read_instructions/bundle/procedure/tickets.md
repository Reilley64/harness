# Dependency-linked tickets

Read the source spec's full body and comments, prior decisions and current code. Use repository domain language. Draft tracer-bullet vertical slices, each verifiable end to end in a fresh assignment; prefactor only where needed. For a genuinely wide mechanical refactor, use expand-contract with explicit migration and final verification dependencies.

Present a numbered breakdown: title, genuine blocking edges and end-to-end deliverable. Ask about granularity, dependencies and merges/splits; iterate until the human approves the breakdown before publishing.

Publish one ticket per slice in blocker-first order using the configured tracker. Use native parent/blocker links where supported, otherwise the documented repository fallback. For local-file trackers follow the repository layout with one numbered file per ticket and explicit dependency identifiers. Include acceptance criteria and apply the actual ready vocabulary. Read back identifiers/relationships and report unsupported operations honestly.

Do not close/modify the parent merely because tickets exist. Publication/readiness and implementation approval are separate. Return approved spec/ticket/revision pointers for the later Chief of Staff path.
