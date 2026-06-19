import { test, expect, type Page } from "@playwright/test";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "";
const ADMIN_EMAIL = process.env.SODMIN_E2E_ADMIN_EMAIL || "";
const ADMIN_PASSWORD = process.env.SODMIN_E2E_ADMIN_PASSWORD || "";
const POLICY_ID = process.env.SODMIN_E2E_POLICY_ID || "";
const REQUIRED_SCOPE = process.env.SODMIN_E2E_POLICY_REQUIRED_SCOPE || "";

async function login(page: Page) {
  await page.goto(`${BASE_URL}/login`);
  await page.getByLabel(/email/i).fill(ADMIN_EMAIL);
  await page.getByLabel(/password/i).fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in|log in|login/i }).click();
  await expect(page.locator("nav")).toBeVisible({ timeout: 15_000 });
}

test.describe("policy editor guardrail visibility", () => {
  test.beforeAll(async () => {
    if (!BASE_URL || !ADMIN_EMAIL || !ADMIN_PASSWORD || !POLICY_ID) {
      test.skip(
        true,
        "SODMIN_E2E_BASE_URL / _ADMIN_EMAIL / _ADMIN_PASSWORD / _POLICY_ID must be set",
      );
    }
  });

  test("surfaces approval evidence, audit trail, and permission context", async ({
    page,
  }) => {
    await login(page);
    await page.goto(`${BASE_URL}/policy`);
    await expect(page.getByRole("heading", { name: /policy|策略/i })).toBeVisible();

    const row = page.locator("tr").filter({ hasText: POLICY_ID }).first();
    await expect(row).toBeVisible({ timeout: 15_000 });
    await expect(row).toContainText(/approval evidence|审批证据/i);
    await expect(row).toContainText(/audit|审计/i);

    await row.getByRole("button", { name: /edit|编辑/i }).click();
    const panel = page.getByTestId("policy-guardrail-panel");
    await expect(panel).toBeVisible();
    await expect(panel.getByTestId("policy-approval-evidence").first()).toBeVisible();
    await expect(panel.getByTestId("policy-audit-entry").first()).toBeVisible();

    if (REQUIRED_SCOPE) {
      await expect(panel).toContainText(REQUIRED_SCOPE);
    }
  });
});
