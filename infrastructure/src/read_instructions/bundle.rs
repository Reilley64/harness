pub(super) fn read() -> Vec<domain::Instruction> {
    let entries: &[(&str, &str, &[&str], &[&str])] = &[
        ("contract", include_str!("bundle/contract.md"), &[], &[]),
        (
            "role/griller",
            include_str!("bundle/role/griller.md"),
            &["contract"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/grill-with-docs/SKILL.md",
            ],
        ),
        (
            "procedure/grilling",
            include_str!("bundle/procedure/grilling.md"),
            &["supplement/domain-modeling"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/grill-with-docs/SKILL.md",
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/grilling/SKILL.md",
            ],
        ),
        (
            "supplement/domain-modeling",
            include_str!("bundle/supplement/domain-modeling.md"),
            &["reference/glossary-format", "reference/adr-format"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/domain-modeling/SKILL.md",
            ],
        ),
        (
            "reference/glossary-format",
            include_str!("bundle/reference/glossary-format.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/domain-modeling/GLOSSARY-FORMAT.md",
            ],
        ),
        (
            "reference/adr-format",
            include_str!("bundle/reference/adr-format.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/domain-modeling/ADR-FORMAT.md",
            ],
        ),
        (
            "procedure/specification",
            include_str!("bundle/procedure/specification.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/to-spec/SKILL.md",
            ],
        ),
        (
            "procedure/tickets",
            include_str!("bundle/procedure/tickets.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/to-tickets/SKILL.md",
            ],
        ),
        (
            "role/setup",
            include_str!("bundle/role/setup.md"),
            &["contract", "procedure/setup"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/SKILL.md",
            ],
        ),
        (
            "procedure/setup",
            include_str!("bundle/procedure/setup.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/SKILL.md",
            ],
        ),
        (
            "role/research",
            include_str!("bundle/role/research.md"),
            &["contract", "procedure/research"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/research/SKILL.md",
            ],
        ),
        (
            "procedure/research",
            include_str!("bundle/procedure/research.md"),
            &["procedure/handoff"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/research/SKILL.md",
            ],
        ),
        (
            "role/chief-of-staff",
            include_str!("bundle/role/chief-of-staff.md"),
            &["contract", "procedure/coordination", "procedure/handoff"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/in-progress/chief-of-staff/SKILL.md",
            ],
        ),
        (
            "procedure/coordination",
            include_str!("bundle/procedure/coordination.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/in-progress/chief-of-staff/SKILL.md",
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement-spec/SKILL.md",
            ],
        ),
        (
            "role/implementer",
            include_str!("bundle/role/implementer.md"),
            &["contract", "procedure/implementation"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement/SKILL.md",
            ],
        ),
        (
            "procedure/implementation",
            include_str!("bundle/procedure/implementation.md"),
            &["supplement/verification", "procedure/handoff"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement/SKILL.md",
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement-spec/SKILL.md",
            ],
        ),
        (
            "supplement/verification",
            include_str!("bundle/supplement/verification.md"),
            &["reference/deep-modules"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/tdd/SKILL.md",
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/tdd/tests.md",
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/tdd/mocking.md",
            ],
        ),
        (
            "reference/deep-modules",
            include_str!("bundle/reference/deep-modules.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/codebase-design/SKILL.md",
            ],
        ),
        (
            "role/reviewer",
            include_str!("bundle/role/reviewer.md"),
            &["contract", "procedure/review"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/code-review/SKILL.md",
            ],
        ),
        (
            "procedure/review",
            include_str!("bundle/procedure/review.md"),
            &[
                "procedure/review-standards",
                "procedure/review-spec",
                "procedure/handoff",
            ],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/code-review/SKILL.md",
            ],
        ),
        (
            "procedure/review-standards",
            include_str!("bundle/procedure/review-standards.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/code-review/SKILL.md",
            ],
        ),
        (
            "procedure/review-spec",
            include_str!("bundle/procedure/review-spec.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/code-review/SKILL.md",
            ],
        ),
        (
            "role/retro",
            include_str!("bundle/role/retro.md"),
            &["contract", "procedure/retro", "procedure/handoff"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/retro/SKILL.md",
            ],
        ),
        (
            "procedure/retro",
            include_str!("bundle/procedure/retro.md"),
            &["supplement/writing-for-agents"],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/retro/SKILL.md",
            ],
        ),
        (
            "procedure/handoff",
            include_str!("bundle/procedure/handoff.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/handoff/SKILL.md",
            ],
        ),
        (
            "supplement/writing-for-agents",
            include_str!("bundle/supplement/writing-for-agents.md"),
            &[],
            &[
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/writing-for-agents/SKILL.md",
                "https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/writing-for-agents/SKILL-MECHANICS.md",
            ],
        ),
        (
            "decision/decider",
            include_str!("bundle/decision/decider.md"),
            &[],
            &[],
        ),
    ];
    entries
        .iter()
        .map(|(id, body, requires, upstream)| domain::Instruction {
            id: (*id).into(),
            version: if matches!(*id, "role/griller" | "procedure/coordination") {
                "harness-native-selection-1 (Harness contract)"
            } else if *id == "procedure/setup" {
                "harness-setup-2 (adapted from upstream 1.3.2)"
            } else if upstream.is_empty() {
                "harness-main-flow-1 (Harness contract)"
            } else {
                "harness-main-flow-1 (adapted from upstream 1.3.2)"
            }
            .into(),
            source: format!("bundle:{id}"),
            upstream: upstream.iter().map(|value| (*value).into()).collect(),
            requires: requires.iter().map(|value| (*value).into()).collect(),
            body: (*body).into(),
        })
        .collect()
}
