# Capability preservation — authority-commit migration

sodmin is the Arkret server administration UI. The authority-commit migration
removed Seal, protocol-level Cell, causal registers and the Notary from the
protocol, so every admin surface that was built on those objects had to be
re-pointed at the surface that now carries the same operational fact: the
authority-signed `RealmCommit` stream of a Realm's single current governing
Station (`sync/authority-commit-log.md`).

This table is the per-entry record required by the migration task: every admin
entry point that existed before the migration, where its capability lives now,
and which test covers it.

## Migrated

| Admin entry point (before) | Implementation now | Test |
| --- | --- | --- |
| `/realms/:realm_id/seal-chain` — "confirmed Seal head" for a Realm | `/realms/:realm_id/authority` — `src/pages/realms/authority.rs`, "Realm commit stream head" card: `stream_position`, head `RealmCommitId`, genesis Event and genesis `RealmCommit`. Data from `src/api/authority.rs` (`POST /_arkret/open/realm-authority/bundle`). | `tests/e2e/a11y.spec.ts`, `tests/e2e/keyboard-strands.spec.ts` and `tests/e2e/high-traffic-pages.spec.ts` cover the navigable admin route set; no route-specific spec existed for the Seal chain page before the migration either. |
| `/realms/:realm_id/notary` — the Realm's fixed notary signer | `/realms/:realm_id/authority`, "Current governing Station" card: `current_service_id`, `current_generation`, `last_handoff_ref`, nonce-bound assertion and its expiry. The Realm's signer is now the current governing Station service key, identified by authority generation. | as above |
| Sidebar section "Seal control" with three Realm deep links | Sidebar section "Realm authority" with one deep link (`src/components/sidebar.rs`). | as above |
| `sodmin-smoke` reachability checks for `realms/{id}/notary`, `realms/{id}/seal-chain`, `realms/{id}/bottom` | `src/bin/sodmin_smoke.rs` now probes `realms/{id}/links` and `realms/{id}/organizations`, which are live Realm-scoped admin reads. The capability (post-deploy admin reachability + auth probe) is unchanged. | `cargo test --bin sodmin-smoke --features smoke` (unit tests in the same file) |

## Newly exposed

| Capability | Implementation | Test |
| --- | --- | --- |
| Continuous old/new double-signed authority handoff chain, genesis → current generation | `src/pages/realms/authority.rs`, `HandoffRow` table: generations, from/to Station, change `RealmCommit`, snapshot ref, and both signature verification methods. A gap or a missing acceptance signature is the operator-visible symptom of an unverifiable Realm authority. | covered by the admin route specs above |

## Removed with justification

| Admin entry point | Why it has no successor |
| --- | --- |
| `/seal/bottom` — "Data conflict diagnostics" (`src/pages/seal_bottom.rs`, `BottomEntry`, `BottomKind`) | The page rendered ⊥ (bottom) values of protocol-level Cells, i.e. conflicting joins of a causal register. Protocol-level Cells, CRDT joins and causal registers are all deleted: typed reducers now run in `RealmCommit` position order on one stream, so a later legal Event overwrites the same target rather than producing a conflicting pair. There is no residual product semantics — the page displayed a protocol object that no longer exists, and the page conferred no permission and drove no workflow. The operational hazard that replaces it, governing-Station equivocation (two differently-signed Commits at the same `(realm_id, stream_ref, authority_generation, stream_position)`), is a *consumer-side* freeze obligation; the Station sodmin administers is itself the authority and cannot meaningfully report its own equivocation, and soland exposes no admin surface for it. If such a surface is added later it belongs beside the authority view, not as a revived Bottom page. |

## Unchanged product capability

Nothing else in sodmin changed. Actors, Realms, Spaces, media, federation,
devices, capabilities, handles, key backup, audit, invite tokens, policy, server
status, service routes, hardening and the whole coauth admin section keep their
routes, APIs and i18n keys.

## Protocol type ownership

`src/api/authority.rs` and `src/pages/realms/authority.rs` use
`arkret_wire::{AuthorityBundleRequest, RealmAuthorityBundle, RealmAuthorityTransition}`
directly. sodmin defines no protocol type of its own; `src/types/seal.rs`, which
re-exported the deleted `soland_contracts::admin::seal` DTOs, is gone.

## External blockers at the time of this change

`cargo check` for this crate stops before sodmin's own code compiles:

- `soland-contracts` fails on `unresolved import arkret_models_collaboration::events_payloads::MediaServiceFocus`. The type is still defined in `arkret-spec`
  (`event-payload.schema.json#/$defs/realm_media_service_payload`) but is absent
  from the SDK's generated Rust surface.
- With `--all-features`, `arkret-models-collaboration` itself fails in
  `governance/audit.rs` on `ReasonCode::RELAXED_WINDOW_EXCEEDS_CEILING`,
  `ProfileId::{ATTESTED_AUDIT_E2EE_V1, DISCLOSED_AUDIT_E2EE_V1}` and
  `SchemaId::{AUDIT_RYW_RECEIPT_V1, AUDIT_RELEASE_ATTESTATION_V1}` — the
  audited-E2EE removal is landed in the generated constants but not yet in the
  hand-written module.

Both are upstream (arkret-rust-sdk / soland) and neither is caused by this
change. The Playwright e2e suite needs a running soland + coauth stack and a
built SPA, so it cannot run while the SPA cannot build.
