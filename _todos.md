# sodmin Active TODO

> 更新日期: 2026-04-29
> 范围: Contrix Principal Server + coauth 管理员 Web UI。`sodmin` 只消费稳定 admin/API contract，不在前端重实现协议 reducer 或授权判定。

## 0. 当前边界

- 当前代码已有 Contrix 页面和 API 模块: actors、spaces、devices、capabilities、federation、applets、agents、reports、invite tokens、audit、policy、media、coauth。
- 历史 README、部分 i18n 和 e2e 仍保留 Palpo/Matrix/Pasion 语义，容易误导测试与部署。
- `_restapi.json` 是手写 admin contract 草案，后续应由 `soland` 与 `coauth` OpenAPI 生成或校验。
- 当前主要缺口: contract source of truth、真实后端联调、权限/审计 UX、深层 legacy 文案清理、E2E stack 更新。

## P0: API Contract Source of Truth

目标: `sodmin` 不维护独立私有 REST 规范，而是消费 `soland` / `coauth` 的 OpenAPI。

- [ ] 替换 `_restapi.json` 工作流:
  - [ ] 从 `soland` 生成 Principal Server Admin OpenAPI。
  - [ ] 从 `coauth` 生成 Auth / Account Admin OpenAPI。
  - [ ] 生成 Rust/WASM API types 或校验现有手写 types。
  - [ ] 保留 `_restapi.json` 仅作为迁移说明或删除。
- [x] 统一 error envelope:
  - [x] not_found。
  - [x] unauthenticated。
  - [x] capability_denied。
  - [x] rate_limited + retry metadata。
  - [x] temporarily_unavailable。
  - [x] validation/schema errors。
- [ ] Pagination/filter contract:
  - [ ] cursor pagination。
  - [ ] stable sort。
  - [ ] filter validation errors。
  - [ ] stale frontier reporting。
- [x] API client hygiene:
  - [x] no token in URL。
  - [x] `X-Contrix-Request-Id` per mutation。
  - [x] `Idempotency-Key` for create/update where supported。
  - [x] Retry-After display。
  - [x] log redaction。

并行性: Principal Server API、coauth API、error/pagination/client hygiene 可并行；type generation strategy 需要先定。

## P0: Authentication, Session and Admin Authorization

- [ ] coauth OAuth2 login:
  - [x] authorization code + PKCE。
  - [x] admin scope request。
  - [x] state validation。
  - [x] nonce generation and authorization request binding。
  - [ ] ID token nonce validation when coauth returns ID tokens。
  - [x] callback error handling。
  - [x] logout/revoke。
- [ ] Token storage:
  - [ ] browser storage threat model。
  - [x] refresh flow or explicit re-auth policy。
  - [x] token expiry UI。
  - [x] no token in local logs/errors。
- [ ] Admin scope model:
  - [x] `urn:coauth:admin` for coauth routes。
  - [x] `urn:contrix:admin:*` for Principal Server routes。
  - [ ] read-only vs mutation permission display。
  - [ ] route guard by feature/profile/scope。
- [ ] High-risk action UX:
  - [ ] require confirmation。
  - [ ] require reason。
  - [ ] show actor/device/admin identity。
  - [ ] show audit id after success。
  - [ ] support approval/proposal state where backend requires it。

## P0: Principal Server Management Pages

目标: 所有页面连接真实 endpoint，显示协议上重要的审计/安全字段。

- [ ] Dashboard:
  - [ ] server describe/profile。
  - [ ] storage mode。
  - [ ] sync/index/blob/federation health。
  - [ ] conformance coverage summary。
- [ ] Actors:
  - [ ] list/search。
  - [ ] detail。
  - [ ] devices。
  - [ ] sessions。
  - [ ] DID/handle info。
  - [ ] account lifecycle status when coauth linked。
- [ ] Spaces:
  - [ ] list/search with discoverability filters。
  - [ ] detail。
  - [ ] members。
  - [ ] invites。
  - [ ] policy/discoverability/history visibility。
  - [ ] plaintext_visible_services。
  - [ ] archive/delete with audit reason。
- [ ] Devices:
  - [ ] inventory。
  - [ ] trust state。
  - [ ] revoke cascade preview。
  - [ ] key package status。
- [ ] Capabilities:
  - [ ] grants/delegations/revocations。
  - [ ] resource selector display。
  - [ ] constraint display。
  - [ ] effective permission explanation。
  - [ ] stale frontier/conflict records。
- [ ] Federation:
  - [ ] peers/service DIDs。
  - [ ] transactions。
  - [ ] replay/fork quarantine。
  - [ ] pull/push failures。
  - [ ] verify-actor challenge status。
- [ ] Blob/media:
  - [ ] metadata list。
  - [ ] quota。
  - [ ] access grants。
  - [ ] retention/legal hold。
  - [ ] unsafe media flags。
  - [ ] authenticated download diagnostics without leaking private blob existence。
- [ ] Reports/moderation:
  - [ ] report queue。
  - [ ] quarantine/review actions。
  - [ ] appeal state。
  - [ ] audit trail。
- [ ] Audit:
  - [ ] request id。
  - [ ] actor/device/admin。
  - [ ] target。
  - [ ] operation/commit id。
  - [ ] outcome。
  - [ ] filter/export。

## P0: coauth Management Pages

- [ ] Accounts:
  - [ ] search/list/detail。
  - [ ] lock/disable/erase。
  - [ ] DID bindings。
  - [ ] recovery status。
  - [ ] claim summary。
- [ ] Sessions/devices:
  - [ ] browser sessions。
  - [ ] OAuth2 sessions。
  - [ ] personal access sessions。
  - [ ] revoke/regenerate。
  - [ ] device binding and risk/MFA state。
- [ ] Upstream providers:
  - [ ] list/create/update/delete。
  - [ ] enable/disable。
  - [ ] discovery/JWKS diagnostics。
  - [ ] claim mapping preview。
- [ ] OAuth2 clients:
  - [ ] client list/detail。
  - [ ] redirect URI validation。
  - [ ] localized metadata editor。
  - [ ] admin scopes review。
- [ ] Registration tokens:
  - [ ] create/revoke。
  - [ ] expiry/max use/pending/completed。
  - [ ] audit reason。
- [ ] Notifications:
  - [ ] channels。
  - [ ] templates。
  - [ ] publish/sync status。
  - [ ] test send with redaction。
- [ ] Policy/claims:
  - [ ] policy dry-run。
  - [ ] issue/revoke claim。
  - [ ] revocation status display。
  - [ ] signed decision viewer。

## P0: Legacy Palpo/Matrix/Pasion Cleanup

- [ ] README / README.zh:
  - [x] rename Palpo Admin -> Contrix Admin / sodmin。
  - [x] replace Matrix users/rooms/media language with actors/spaces/blob。
  - [x] replace Pasion references with coauth where appropriate。
  - [ ] update Docker/example stack docs。
- [ ] i18n:
  - [x] remove `Palpo Admin` title from default visible chrome。
  - [x] remove Matrix-specific subtitles from default visible labels。
  - [x] rename rooms/users to spaces/actors in default visible labels。
  - [ ] keep legacy compatibility labels only in compatibility sections。
- [ ] e2e:
  - [ ] replace Matrix scopes helpers with Contrix/coauth scopes。
  - [ ] replace Palpo/Pasion fixture names。
  - [ ] remove Element-specific smoke from default sodmin suite。
  - [ ] add Contrix stack fixtures。
- [ ] Types/errors:
  - [x] rename `MatrixError` to Contrix/Admin error type。
  - [ ] remove `_matrix` endpoint assumptions。
  - [x] align error codes with Contrix API convention。

## P1: End-to-End Stack and CI

- [ ] Local compose stack:
  - [ ] soland。
  - [ ] coauth。
  - [ ] starid。
  - [ ] floria mock mode。
  - [ ] sodmin。
  - [ ] PostgreSQL。
- [ ] Playwright coverage:
  - [ ] login/logout。
  - [ ] dashboard loads profile。
  - [ ] actors list/detail。
  - [ ] spaces list/detail/member mutation。
  - [ ] capability explanation。
  - [ ] federation quarantine page。
  - [ ] blob anti-enumeration diagnostics。
  - [ ] coauth account/session/registration token pages。
  - [ ] audit log after mutation。
- [ ] CI:
  - [ ] `cargo fmt -- --check`。
  - [ ] `cargo check` / Dioxus build。
  - [ ] Playwright component/smoke。
  - [ ] generated API type drift check。
  - [ ] screenshots/artifacts on failure。

## P1: UX, Accessibility and Operations

- [ ] Loading/error empty states use consistent PageShell。
- [ ] Route-level feature gates based on server describe profiles。
- [ ] Keyboard navigation and focus management for destructive dialogs。
- [ ] Screen reader labels for tables/actions。
- [ ] i18n parity for English/Chinese。
- [ ] Audit-sensitive pages default to least data exposure。
- [ ] Time/date formatting stable across locales。
- [ ] Export/download actions warn about sensitive data。

## 本轮验证记录

- [x] 2026-04-29: `cargo fmt --all`。
- [x] 2026-04-29: `cargo check --message-format short`。
- [x] 2026-04-29: `cargo test --message-format short`。
- [x] 2026-04-29: API client unit tests cover query credential rejection, diagnostic URL redaction, mutation idempotency headers, and retry metadata preservation。
- [x] 2026-04-29: OAuth scope unit test covers `urn:coauth:admin` and `urn:contrix:admin:*` while rejecting legacy `urn:cx:admin`。

## Definition of Done

- [ ] Page consumes stable generated or validated API contract。
- [ ] Mutation has confirmation, reason when needed, request id and audit feedback。
- [ ] E2E covers happy path and at least one denied/error path。
- [ ] Legacy Matrix/Palpo/Pasion wording is removed from default Contrix UI/docs。
- [ ] No token, DID private proof, push key or blob secret appears in URL/log/UI diagnostics。
