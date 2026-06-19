import { test, expect, type Page } from "@playwright/test";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "";
const ADMIN_EMAIL = process.env.SODMIN_E2E_ADMIN_EMAIL || "";
const ADMIN_PASSWORD = process.env.SODMIN_E2E_ADMIN_PASSWORD || "";
const PIN_POLICY_ID = process.env.SODMIN_E2E_PIN_POLICY_ID || "";
const PRIVATE_SENTINEL = process.env.SODMIN_E2E_PIN_POLICY_PRIVATE_SENTINEL || "";

async function login(page: Page) {
  await page.goto(`${BASE_URL}/login`);
  await page.getByLabel(/email/i).fill(ADMIN_EMAIL);
  await page.getByLabel(/password/i).fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in|log in|login/i }).click();
  await expect(page.locator("nav")).toBeVisible({ timeout: 15_000 });
}

test.describe("pin policy guardrails", () => {
  test.beforeAll(async () => {
    if (!BASE_URL || !ADMIN_EMAIL || !ADMIN_PASSWORD || !PIN_POLICY_ID) {
      test.skip(
        true,
        "SODMIN_E2E_BASE_URL / _ADMIN_EMAIL / _ADMIN_PASSWORD / _PIN_POLICY_ID must be set",
      );
    }
  });

  test("renders pin policy rows read-only and hides private material", async ({
    page,
  }) => {
    await login(page);
    await page.goto(`${BASE_URL}/policy`);

    const row = page.locator("tr").filter({ hasText: PIN_POLICY_ID }).first();
    await expect(row).toBeVisible({ timeout: 15_000 });
    await expect(row).toContainText(/pin policy|pin 策略/i);
    await expect(row).toContainText(/read-only|只读/i);

    await expect(row.getByRole("button", { name: /delete|删除/i })).toBeDisabled();
    await row.getByRole("button", { name: /view|查看/i }).click();

    const panel = page.getByTestId("pin-policy-safety-panel");
    await expect(panel).toBeVisible();
    await expect(panel).toContainText(/standard api unavailable|标准 api 不可用/i);
    await expect(page.getByRole("button", { name: /save|保存/i })).toHaveCount(0);

    if (PRIVATE_SENTINEL) {
      await expect(page.locator("body")).not.toContainText(PRIVATE_SENTINEL);
    }
  });
});
