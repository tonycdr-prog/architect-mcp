# Stack Pack Expansion Strategy

## Priority
- Next.js App Router
- React/Vite
- Node API
- Postgres
- Supabase
- Expo React Native

## Pack Contract
- Concrete directories for generated repos.
- File rules with trigger, recommendation, path scope, good/bad examples, triggerKind, and detectors.
- Module boundaries and verification expectations that prevent UI -> DB leaks, shared -> feature leaks, route/controller overreach, unsafe secret access, and unverified implementation claims.
- Testing expectations for generated app slices.
- Source metadata from reviewed docs snapshots or llms.txt-style text.

## Promotion Flow
1. Capture local source text or an llms.txt snapshot.
2. Run derive_stack_pack_from_llms_source when a local ingested snapshot exists.
3. Run propose_stack_pack_rules when source text is supplied directly.
4. Run review_stack_pack_candidate.
5. Run analyze_stack_pack_conflicts against existing and candidate packs.
6. Promote only reviewed candidates.
7. Write the pack JSON, update packs/manifest.json, and run validate_stack_packs.

Live web fetching should come after this review workflow, not before it.

## Source-Backed Rule Depth

Rules derived from `llms.txt` sources should include enough provenance for a reviewer to decide whether they are trustworthy:

- source id, URL, snapshot path, fetched timestamp, and SHA-256 hash
- matched evidence headings or links
- concrete detector mapping, not just prose advice
- conflict review against existing stack and foundation packs
- small good/bad examples when the rule will be shown to an agent

Do not promote broad rules that only say "use best practices" or "keep code clean." Convert source material into observable repo boundaries, file rules, verification expectations, or agent instructions.

## Upstream LLM Sources

Known sources are cataloged in `stack-sources/llms-sources.json` and exposed through `discover_llms_sources`.

Use:

```text
discover_llms_sources -> fetch_llms_source -> ingest_llms_txt -> derive_stack_pack_from_llms_source -> review_stack_pack_candidate -> analyze_stack_pack_conflicts -> promote_stack_pack_candidate
```

High-priority sources:

- Next.js: https://nextjs.org/docs/llms.txt
- React: https://react.dev/llms.txt
- Vite: https://vite.dev/llms.txt
- Expo: https://docs.expo.dev/llms.txt
- Supabase: https://supabase.com/llms.txt
- Vercel: https://vercel.com/llms.txt
- Auth0: https://auth0.com/llms.txt
- Stripe: https://docs.stripe.com/llms.txt
- OpenAI Platform: https://platform.openai.com/llms.txt
- Vercel AI SDK: https://ai-sdk.dev/llms.txt
