import { expect, test, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "";
const ADMIN_EMAIL = process.env.SODMIN_E2E_ADMIN_EMAIL || "";
const ADMIN_PASSWORD = process.env.SODMIN_E2E_ADMIN_PASSWORD || "";

async function login(page: Page) {
  await page.goto(`${BASE_URL}/login`);
  await page.getByLabel(/email/i).fill(ADMIN_EMAIL);
  await page.getByLabel(/password/i).fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in|log in|login/i }).click();
  await expect(page.locator("main")).toBeVisible({ timeout: 15_000 });
}

const routes = [
  "/",
  "/devices",
  "/federation",
  "/audit",
  "/policy",
  "/server-status",
];

test.describe("axe accessibility audit", () => {
  test.beforeAll(async () => {
    if (!BASE_URL || !ADMIN_EMAIL || !ADMIN_PASSWORD) {
      test.skip(
        true,
        "SODMIN_E2E_BASE_URL / _ADMIN_EMAIL / _ADMIN_PASSWORD must be set",
      );
    }
  });

  for (const route of routes) {
    test(`${route} has no critical axe violations`, async ({ page }) => {
      await login(page);
      await page.goto(`${BASE_URL}${route}`);
      await expect(page.locator("main")).toBeVisible({ timeout: 10_000 });

      const results = await new AxeBuilder({ page })
        .disableRules(["color-contrast"])
        .analyze();
      const critical = results.violations.filter((violation) =>
        ["critical", "serious"].includes(violation.impact || ""),
      );

      expect(critical).toEqual([]);
    });
  }
});
