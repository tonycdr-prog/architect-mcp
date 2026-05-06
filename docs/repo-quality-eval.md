# Repo Quality Eval

The repo quality eval layer is not reinforcement learning and not a single reward score. It is a structured review layer that helps clients decide whether to ask more questions, fix a plan, block unsafe work, or proceed.

## Fit In The Architecture

The MCP exposes pure tools and structured context:

- `build_quality_requirements_profile`
- `evaluate_repo_plan_quality`
- `audit_generated_repo_quality`
- `suggest_quality_followup_questions`
- `run_repo_quality_eval_scenarios`

The harness or client orchestrator owns the decision loop:

1. Interview the user.
2. Build a requirements profile.
3. Score proposed stack/repo plans.
4. Ask one focused follow-up question when confidence is low.
5. Generate the repo only after hard gates are clear.
6. Audit the generated repo.
7. Fix issues or present the final result with proof.

## Hard Gates

These block or require fixes before generation/completion:

- Committed or hardcoded secrets.
- Required environment variables without `.env.example`.
- CI that only echoes success or does not run meaningful checks.
- Trivial, fake, or placeholder tests.
- Destructive commands without explicit approval.
- Unsafe or overbroad permissions.
- Missing setup instructions.
- Missing or vague `AGENTS.md`.
- Jargon-heavy explanations for non-technical users.

## Rubric Dimensions

The scored dimensions are separate so agents cannot hide weakness behind one high number:

- Requirements fit.
- Simplicity.
- Maintainability.
- Security.
- CI and test quality.
- Documentation quality.
- Agent readiness.
- Suitability for non-technical users.

## Reward-Hacking Guardrails

The evaluator explicitly warns about shallow success signals:

- CI passing is not enough when tests are weak or fake.
- A simple stack is not good if it cannot meet the user's goals.
- A complex stack must be justified by requirements.
- User preference cannot override security or maintainability gates.

Use this layer alongside `/grill-me`, `review_proposed_file_plan`, `review_repo_structure`, `score_agent_artifacts`, and `review_agent_final_response`.
