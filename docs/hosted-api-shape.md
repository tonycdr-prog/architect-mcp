# Hosted API Shape

Hosted mode should stay stateless by default. Clients send explicit snapshots and receive structured decisions; architect-mcp must not read arbitrary server-local paths.

## Principles

- No workspace filesystem access in hosted mode.
- No persistence unless a client enables a future adapter for memory, review history, teams, billing, or analytics.
- Every hosted request should carry enough context for one deterministic review.
- Large inputs should be summarized by clients before submission when possible.
- Secrets must be omitted or redacted before requests reach hosted mode.

## Core Endpoints

These are API-shape targets, not a hosting implementation.

### POST /v1/intake/grill

Input:

- `brief`: project brief object.
- `stackPackIds`: optional stack-pack ids.
- `includeContract`: optional boolean.
- `includeArtifacts`: optional boolean.

Output:

- `/grill-me` readiness result.
- Optional architecture contract and generated artifacts.
- Next focused question when not ready.

### POST /v1/harness/intent

Input:

- `request`: user implementation request.
- `mode`: `strict`, `guided-yolo`, or `full-yolo`.
- `stack`: optional stack hints.
- `repoSignals`: optional file-summary or package hints.

Output:

- Intent interpretation.
- Ambiguity/blast-radius decision.
- Triggered stack guidance.
- Confirmation prompt when required.

### POST /v1/reviews/repo

Input:

- `files`: explicit file summaries supplied by the client.
- `directories`: explicit directory list.
- `contract`: optional architecture contract.
- `baseline`: optional review baseline.
- `mode`: review gate mode.

Output:

- Findings.
- Review report and lifecycle classification.
- Stable codes and confidence levels.

### POST /v1/reviews/session

Input:

- Intent result.
- Pre-edit contract.
- Changed file summaries.
- Verification statuses.
- Final agent response.
- Optional memory proposals.

Output:

- Combined agent-session report.
- Drift, verification honesty, scope, and output-contract findings.

### POST /v1/security/mcp-config

Input:

- MCP config object supplied by the client.
- Optional approved server names.

Output:

- MCP config security findings.
- Pass/warn/fail status.

## Future Adapters

Adapters should be explicit opt-ins:

- GitHub memory PRs.
- Review history and baselines.
- Team policy packs.
- Hosted artifact inbox.
- Billing and usage analytics.

The local MCP tools remain the source of truth for behavior. Hosted API routes should wrap the same domain functions and preserve the same output shapes.
