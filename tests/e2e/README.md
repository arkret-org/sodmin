# tests/e2e — Playwright suite

This directory holds the Playwright e2e tests for sodmin.

## Q2 — happy-path risk-action

`risk-action.spec.ts` drives the full risk-action approval flow:

1. Login at `/login`
2. Navigate to `/coauth/accounts`
3. Open an account detail page
4. Click "Approve risk action"
5. Confirm the ConfirmDialog
6. Assert the audit log surfaces a `risk_action` entry

## Running locally

```bash
# 1. Boot a sodmin dev instance + soland + coauth (manual, today)
#    — typically: `npm run stack:up` in the sodmin repo root.

# 2. Export env vars pointing at the dev instance
export SODMIN_E2E_BASE_URL=http://localhost:8080
export SODMIN_E2E_ADMIN_EMAIL=admin@example.com
export SODMIN_E2E_ADMIN_PASSWORD=changeme
export SODMIN_E2E_ACCOUNT_ID=01HXY...

# 3. From the sodmin repo root, install playwright + browsers if needed
npm install
npx playwright install chromium

# 4. Run the round-26 suite
npx playwright test -c tests/e2e/playwright.config.ts
```

If any of the four env vars is missing, the spec self-skips so a
plain `playwright test` against this config in CI still passes
without a wired dev stack.

## Skip behaviour summary

| Condition                                         | Outcome |
|---------------------------------------------------|---------|
| `SODMIN_E2E_BASE_URL` unset                       | skip    |
| `SODMIN_E2E_ADMIN_EMAIL` / `_PASSWORD` unset      | skip    |
| `SODMIN_E2E_ACCOUNT_ID` unset                     | skip    |
| All env vars set + dev stack running              | run     |

## Q6 — happy-path recovery ticket lifecycle (round 27)

`recovery.spec.ts` walks a recovery ticket through Pending → Approved →
ExecutorRunning → Complete and asserts the audit log shows the
recovery operations:

1. Login at `/login`
2. Navigate to `/coauth/recovery/tickets`
3. Open the target ticket detail page
4. Click "Approve" → confirm the ConfirmDialog
5. Click "Advance" twice (Approved → ExecutorRunning → Complete)
6. Assert the per-ticket audit feed shows `recovery` rows
7. Cross-check the global coauth audit log surfaces a `recovery` entry

Required env vars (in addition to the shared `SODMIN_E2E_BASE_URL` /
`_ADMIN_EMAIL` / `_ADMIN_PASSWORD`):

- `SODMIN_E2E_RECOVERY_TICKET_ID` — a Pending ticket pre-seeded by the
  operator

If unset, the spec self-skips.

## Phase 3 — high-traffic page smoke

`high-traffic-pages.spec.ts` adds authenticated route-level smokes for
the server-ops surfaces (dashboard, actors, spaces, federation,
deactivation, audit, policy, server status). Governance pages
(moderation/applets/agents/directory/circles) were removed from sodmin
in the P3 ops-level consolidation and now live in yougen, so they are no
longer smoked here. These tests use the shared `SODMIN_E2E_BASE_URL`,
`SODMIN_E2E_ADMIN_EMAIL`, and `SODMIN_E2E_ADMIN_PASSWORD` env vars and
self-skip when the local stack is not configured.

## Q9 — happy-path device revoke + cascade audit (round 27)

`device-revoke.spec.ts` revokes a device on a target account, asserts
the row badge transitions to Revoked, and cross-checks that the
coauth audit feed shows both the device-revoke entry AND a
session-grant cascade entry (proving the soland-side cascade-revoke
landed on the canonical audit pipeline):

1. Login at `/login`
2. Navigate to `/coauth/accounts`
3. Open the target account detail page
4. Jump to `/coauth/accounts/:id/devices`
5. Click "Revoke" on the target row
6. Confirm the destructive ConfirmDialog
7. Assert the row badge transitions to "Revoked"
8. Navigate to `/coauth/audit-log` and assert both a device-revoke row
   AND a session-grant cascade row are visible

Required env vars (in addition to the shared trio):

- `SODMIN_E2E_ACCOUNT_ID` — the target account
- `SODMIN_E2E_DEVICE_ID` — a revocable (Active or Stale) device on
  that account

If unset, the spec self-skips.
