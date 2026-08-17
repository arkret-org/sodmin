# tests/e2e — Playwright suite

This directory holds the Playwright e2e tests for sodmin.

Automated CI runs only `npm run test:smoke`, which is limited to the
unauthenticated responsive smoke that needs `SODMIN_E2E_BASE_URL` only.
The authenticated high-risk management specs in this directory are manual
or seeded-stack checks: they require admin credentials plus account, device,
or policy ids and are intentionally not reported as automated CI coverage.

## Happy-path risk-action

`risk-action.spec.ts` drives the full risk-action approval strand:

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

# 4. Run the whole suite
npx playwright test -c tests/e2e/playwright.config.ts
```

If any of the four env vars is missing, the spec self-skips for local
operator convenience. CI does not invoke this manual spec unless a workflow
explicitly provisions those variables and seeded backend data.

## Skip behaviour summary

| Condition                                         | Outcome |
|---------------------------------------------------|---------|
| `SODMIN_E2E_BASE_URL` unset                       | skip    |
| `SODMIN_E2E_ADMIN_EMAIL` / `_PASSWORD` unset      | skip    |
| `SODMIN_E2E_ACCOUNT_ID` unset                     | skip    |
| All env vars set + dev stack running              | run     |

## High-traffic page smoke

`high-traffic-pages.spec.ts` adds authenticated route-level smokes for
the server-ops surfaces (dashboard, actors, spaces, federation,
deactivation, audit, policy, server status). Governance pages
(moderation/applets/agents/directory/circles) were removed from sodmin
in the ops-level consolidation and now live in inkson, so they are no
longer smoked here. These tests use the shared `SODMIN_E2E_BASE_URL`,
`SODMIN_E2E_ADMIN_EMAIL`, and `SODMIN_E2E_ADMIN_PASSWORD` env vars and
self-skip when the local stack is not configured.

## Happy-path device revoke + cascade audit

`device-revoke.spec.ts` revokes a device on a target account, asserts
the row badge transitions to Revoked, and cross-checks that the
coauth audit feed shows both the device-revoke entry AND a
session-grant cascade entry (proving the issuer-ledger lifecycle mutation
landed on the canonical coauth audit pipeline):

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

## Policy editor guardrail visibility

`policy-audit.spec.ts` verifies that `/policy` surfaces policy guardrail
metadata from a pre-seeded policy document:

1. Login at `/login`
2. Navigate to `/policy`
3. Find the row identified by `SODMIN_E2E_POLICY_ID`
4. Assert the row exposes approval evidence and audit-trail indicators
5. Open the edit modal and assert approval evidence / audit-trail details render
6. If `SODMIN_E2E_POLICY_REQUIRED_SCOPE` is set, assert that scope is visible

Required env vars (in addition to the shared trio):

- `SODMIN_E2E_POLICY_ID` — policy document with `approval_evidence[]` and
  `audit_trail[]` in its payload/resource metadata
- `SODMIN_E2E_POLICY_REQUIRED_SCOPE` — optional exact required-scope text to
  assert

If the policy id is unset, the spec self-skips.
