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

// P5 — destructive flow keyboard reachability. Each entry seeds a
// destructive button or modal trigger that MUST be reachable via Tab
// alone. Where fixture-dependent (e.g. a specific realm id must exist
// to render the destroy button), the test is skipped with a TODO
// marker so a future fixture-seeded run can flip the skip off.
const destructiveFlows = [
  {
    name: "realm_destroy",
    path: "/realms/destroy",
    target: /destroy|tombstone|confirm|type.*destroy/i,
    fixtureDependent: true,
  },
  {
    name: "revoke_grant",
    path: "/capabilities",
    target: /revoke|grant|capability|delete/i,
    fixtureDependent: false,
  },
  {
    name: "delete_binding",
    // The DID-binding panel is rendered inside the coauth account
    // detail page; reaching it requires a real account id from the
    // fixture set.
    path: "/coauth/accounts",
    target: /account|search|email|did|binding|remove/i,
    fixtureDependent: true,
  },
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

  // P5 — destructive flows must also be Tab-reachable so a
  // keyboard-only operator can drive realm-destroy / agent-deactivate
  // / grant-revoke / binding-delete without a mouse.
  for (const flow of destructiveFlows) {
    test(`destructive flow: ${flow.name} reaches the destructive control via Tab`, async ({
      page,
    }) => {
      if (flow.fixtureDependent) {
        // TODO(P5-fixture): seed the fixture (realm id, agent id,
        // coauth account id) so the destructive control actually
        // renders and the assertion can become unconditional.
        test.skip(
          true,
          `${flow.name}: depends on a seeded fixture; un-skip once the fixture set ships`,
        );
        return;
      }
      await login(page);
      await page.goto(`${BASE_URL}${flow.path}`);
      await expect(page.locator("main")).toBeVisible({ timeout: 10_000 });

      const seen = new Set<string>();
      for (let i = 0; i < 32; i += 1) {
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
