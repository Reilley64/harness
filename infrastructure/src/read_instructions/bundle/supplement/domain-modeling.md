# Domain modeling

Challenge terms that conflict with the repository glossary immediately. Propose precise canonical language for overloaded concepts. Stress-test relationships with concrete edge cases and cross-check claims against source.

Follow the repository's domain-document convention first. A root GLOSSARY-MAP.md selects multiple contexts; otherwise use the root GLOSSARY.md. Ask if the context is unclear. Create glossary/ADR artifacts lazily when there is actual resolved content; their absence alone is not Setup failure.

Capture resolved terms inline under the permitted repository scope. Keep glossary definitions free of implementation details; use reference/glossary-format. Offer an ADR only when the choice is hard to reverse, surprising without context and a real trade-off. Use reference/adr-format, adapted to the repository's actual ADR location (including tracker pages).

If the required domain convention has not been captured, ask to resolve it before writing. Human domain decisions and repository write authority remain distinct from this procedure.
