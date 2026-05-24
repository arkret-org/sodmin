import { expect, test, type Page } from "@playwright/test";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "";
const ADMIN_EMAIL = process.env.SODMIN_E2E_ADMIN_EMAIL || "";
const ADMIN_PASSWORD = process.env.SODMIN_E2E_ADMIN_PASSWORD || "";

async function login(page: Page) {
  await page.goto(`${BASE_URL}/login`);
  await page.getByLabel(/email/i).fill(ADMIN_EMAIL);
  await page.getByLabel(/password/i).fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in|log in|login/i }).click();
  await expect(page.locator("nav")).toBeVisible({ timeout: 15_000 });
}

const keyboardRoutes = [
  { name: "devices", path: "/devices", target: /search|refresh|revoke/i },
  { name: "peers", path: "/federation", target: /search|refresh|add|peer/i },
  { name: "audit", path: "/audit", target: /actor|action|target|refresh/i },
  { name: "policy", path: "/policy", target: /name|type|scope|enabled|create/i },
];

test.describe("keyboard-only management flows", () => {
  test.beforeAll(async () => {
    if (!BASE_URL || !ADMIN_EMAIL || !ADMIN_PASSWORD) {
      test.skip(
        true,
        "SODMIN_E2E_BASE_URL / _ADMIN_EMAIL / _ADMIN_PASSWORD must be set",
      );
    }
  });

  for (const flow of keyboardRoutes) {
    test(`${flow.name} exposes reachable focus targets`, async ({ page }) => {
      await login(page);
      await page.goto(`${BASE_URL}${flow.path}`);
      await expect(page.locator("main")).toBeVisible({ timeout: 10_000 });

      const seen = new Set<string>();
      for (let i = 0; i < 24; i += 1) {
        await page.keyboard.press("Tab");
        const label = await page.evaluate(() => {
          const el = document.activeElement as HTMLElement | null;
          return [
            el?.getAttribute("aria-label"),
            el?.getAttribute("name"),
            el?.textContent,
            el?.getAttribute("placeholder"),
          ]
            .filter(Boolean)
            .join(" ");
        });
        if (label) {
          seen.add(label);
        }
      }

      expect([...seen].join(" ")).toMatch(flow.target);
    });
  }
});
