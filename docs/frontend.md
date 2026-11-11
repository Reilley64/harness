# Frontend

The existing Bun/React/Ink launcher is the product UI, not a web application.
This document routes interaction tasks; [ACP](acp-presentation.md) owns wire
contracts and feature documents own runtime policy.

- [Use conversations, the prompt and navigation](frontend/conversations.md).
- [Configure providers, models, MCP and inspect work](frontend/configuration.md).
- [Develop the frontend](frontend/implementation.md): ownership, tools and deferred integrations.

Native byte probes do not establish human visual acceptance. Terminal input uses
the approved bounded 150 ms legacy Escape policy and a single shared input owner.
See the [input contract](frontend/implementation.md#terminal-input) for timer,
framing and unsupported-fragmentation limits.

## MCP configuration

See [MCP configuration](frontend/configuration.md#mcp-configuration).

## Atomic design

See [Atomic design](frontend/implementation.md#atomic-design).

## Bun

See [Bun](frontend/implementation.md#bun).

## Provider connections

See [Provider connections](frontend/configuration.md#provider-connections).

## Model picker

See [Model picker](frontend/configuration.md#model-picker).

## Native Decider controls

See [Native Decider controls](frontend/configuration.md#native-decider-controls).

## Conversations

See [Conversations](frontend/conversations.md#conversations).

## Model permission settings

See [Model permission settings](frontend/configuration.md#model-permission-settings).

## Agent contexts and creation-map navigation

See [Agent contexts and creation-map navigation](frontend/conversations.md#agent-contexts-and-creation-map-navigation).

## Instruction provenance

See [Instruction provenance](frontend/configuration.md#instruction-provenance).

## Approved work and CoS

See [Approved work and CoS](frontend/configuration.md#approved-work-and-cos).

## Repository Setup

See [Repository Setup](frontend/configuration.md#repository-setup).

## TanStack AI

See [TanStack AI](frontend/implementation.md#tanstack-ai).
