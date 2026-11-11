# Deep modules

A module earns its interface by hiding substantial behavior. The interface includes invariants, ordering, errors and configuration as well as signatures. A seam is the public location where an adapter can vary behavior.

Prefer leverage for callers and locality for maintainers. Apply the deletion test: if deleting a wrapper removes complexity instead of moving it back to callers, remove it. Verify observable behavior at public seams; keep collaborators private. Repository architecture and naming rules take precedence over generic design heuristics.
