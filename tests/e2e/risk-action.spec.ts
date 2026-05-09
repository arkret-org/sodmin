// Round 26 Q2 — happy-path risk-action approval Playwright e2e.
//
// Drives the sodmin SPA against a sodmin dev instance + a soland dev
// stack. The test is intentionally narrow: it boots the SPA, signs the
// admin in, navigates to the accounts list, opens an account detail
// page, kicks off the risk-action flow, and asserts that an audit row
// appears for the action.
//
// This file co-exists with the legacy `e2e/` test suite (which targets
// the older Pasion/Palpo example stack). New round-26+ specs land
// under `tests/e2e/` so we can break cleanly from the legacy harness
// without disturbing it.
//
// Skip behaviour: if the env vars `SODMIN_E2E_BASE_URL`,
// `SODMIN_E2E_ADMIN_EMAIL`, `SODMIN_E2E_ADMIN_PASSWORD`, or
// `SODMIN_E2E_ACCOUNT_ID` are unset, the test calls `test.skip()` so a
// CI environment that hasn't wired the dev stack still passes. The
// expectation is that an operator runs the local dev stack manually
// (or via `npm run stack:up`), exports the env vars, then invokes
// `npx playwright test -c tests/e2e/playwright.config.ts`.

import { test, expect } from "@playwright/test";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "";
const ADMIN_EMAIL = process.env.SODMIN_E2E_ADMIN_EMAIL || "";
const ADMIN_PASSWORD = process.env.SODMIN_E2E_ADMIN_PASSWORD || "";
const ACCOUNT_ID = process.env.SODMIN_E2E_ACCOUNT_ID || "";

test.describe("Q2 risk-action happy-path", () => {
  test.beforeAll(async () => {
    if (!BASE_URL || !ADMIN_EMAIL || !ADMIN_PASSWORD || !ACCOUNT_ID) {
      test.skip(
        true,
        "SODMIN_E2E_BASE_URL / _ADMIN_EMAIL / _ADMIN_PASSWORD / _ACCOUNT_ID must be set",
      );
    }
  });

  test("approves a risk-action and surfaces an audit entry", async ({ page }) => {
    // 1. Navigate to login
    await page.goto(`${BASE_URL}/login`);
    await expect(page).toHaveURL(/\/login/);

    // 2. Sign the admin in
    await page.getByLabel(/email/i).fill(ADMIN_EMAIL);
    await page.getByLabel(/password/i).fill(ADMIN_PASSWORD);
    await page.getByRole("button", { name: /sign in|log in|login/i }).click();

    // 3. Wait until the dashboard / sidebar mounts
    await expect(page.locator("nav")).toBeVisible({ timeout: 15_000 });

    // 4. Navigate to the accounts list
    await page.goto(`${BASE_URL}/coauth/accounts`);
    await expect(page.getByRole("heading", { name: /accounts/i })).toBeVisible();

    // 5. Open the target account detail page
    await page.goto(`${BASE_URL}/coauth/accounts/${ACCOUNT_ID}`);
    await expect(page.getByText(ACCOUNT_ID)).toBeVisible({ timeout: 10_000 });

    // 6. Trigger a risk-action proposal (approve flow). The button copy
    //    is "Approve risk action" / 批准风控操作 — we match on either.
    const approveBtn = page.getByRole("button", {
      name: /approve risk action|批准风控操作/i,
    });
    await expect(approveBtn).toBeVisible({ timeout: 10_000 });
    await approveBtn.click();

    // 7. Confirm the destructive ConfirmDialog
    const confirmBtn = page.getByRole("button", { name: /^confirm$|批准|approve$/i });
    await expect(confirmBtn).toBeVisible({ timeout: 5_000 });
    await confirmBtn.click();

    // 8. Toast surfaces — either success or "endpoint not yet wired".
    //    Both are acceptable round-26 outcomes; the audit log assertion
    //    below is the real check.
    await expect(
      page.getByText(/risk action recorded|approved|not yet wired/i),
    ).toBeVisible({ timeout: 10_000 });

    // 9. Navigate to the audit log and expect the entry. The
    //    risk-action approval should leave a row whose action contains
    //    "risk_action".
    await page.goto(`${BASE_URL}/coauth/audit`);
    await expect(page.getByText(/risk_action/i).first()).toBeVisible({
      timeout: 15_000,
    });
  });
});
