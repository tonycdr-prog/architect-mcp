# Stack Packs

Stack packs are source-backed architecture rules for specific frameworks and platforms. They keep review guidance concrete without turning the core work gate into a large public onboarding surface.

## Current Packs

Source-backed packs exist for Hono, Zod, Vitest, Auth0, Stripe, Expo, and Vercel AI SDK. Pack JSON lives under `packs/`, and foundation pack JSON lives under `foundation-packs/`.

## Authoring Contract

Candidate stack packs should include:

- Source metadata and provenance.
- Concrete rules with stable finding codes.
- Trigger terms and detector metadata.
- Module boundaries and examples.
- Tests and agent instructions.

## Promotion Flow

```text
discover_llms_sources
-> fetch_llms_source
-> ingest_llms_txt
-> derive_stack_pack_from_llms_source
-> review_stack_pack_candidate
-> analyze_stack_pack_conflicts
-> promote_stack_pack_candidate
-> promote_stack_pack_to_files
-> validate_stack_packs
```

`promote_stack_pack_to_files` is local-only because it can write `packs/<id>.json` and `packs/manifest.json` updates. Hosted clients should use the non-writing candidate and review tools.

## Coverage

Use `score_stack_packs` and `stack_pack_coverage_matrix` to review source quality, detector coverage, examples, tests, and documentation status.

The detailed strategy is preserved in [Stack Pack Strategy](./stack-pack-strategy.md).
