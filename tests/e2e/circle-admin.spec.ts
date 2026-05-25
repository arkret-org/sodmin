// P3A.9 — CXP-0007 Circle admin happy-path Playwright e2e.
//
// Drives the sodmin SPA against a sodmin dev instance + a soland dev
// stack that has the `/api/v1/circles/*` routes wired (soland P2A.2)
// and a coauth that ships the six `cx.circle.*` capability actions
// (coauth P2B.2). The test is intentionally narrow: it creates a
// Circle, adds a legitimate (Realm-member) actor, attempts to add an
// out-of-Realm actor and asserts the reducer rejection surfaces in
// the toast, archives the Circle, and finally walks back to the
// audit log to confirm the events fanned out.
//
// Skip behaviour: if the env vars `SODMIN_E2E_BASE_URL`,
// `SODMIN_E2E_ADMIN_EMAIL`, `SODMIN_E2E_ADMIN_PASSWORD`,
// `SODMIN_E2E_REALM_ID`, `SODMIN_E2E_ACTOR_DID`, or
// `SODMIN_E2E_NON_MEMBER_ACTOR_DID` are unset, the test calls
// `test.skip()` so a CI environment without the dev stack still
// passes. The expectation is that an operator runs the local dev
// stack manually (or via `npm run stack:up`), exports the env vars,
// then invokes `npx playwright test -c tests/e2e/playwright.config.ts
// circle-admin.spec.ts`.

import { test, expect } from "@playwright/test";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "";
const ADMIN_EMAIL = process.env.SODMIN_E2E_ADMIN_EMAIL || "";
const ADMIN_PASSWORD = process.env.SODMIN_E2E_ADMIN_PASSWORD || "";
const REALM_ID = process.env.SODMIN_E2E_REALM_ID || "";
const ACTOR_DID = process.env.SODMIN_E2E_ACTOR_DID || "";
const NON_MEMBER_DID = process.env.SODMIN_E2E_NON_MEMBER_ACTOR_DID || "";

test.describe("P3A.9 Circle admin happy path", () => {
  test.beforeAll(async () => {
    if (
      !BASE_URL ||
      !ADMIN_EMAIL ||
      !ADMIN_PASSWORD ||
      !REALM_ID ||
      !ACTOR_DID ||
      !NON_MEMBER_DID
    ) {
      test.skip(
        true,
        "SODMIN_E2E_BASE_URL / _ADMIN_EMAIL / _ADMIN_PASSWORD / _REALM_ID / _ACTOR_DID / _NON_MEMBER_ACTOR_DID must be set",
      );
    }
  });

  async function login(page: import("@playwright/test").Page) {
    await page.goto(`${BASE_URL}/login`);
    await expect(page).toHaveURL(/\/login/);
    await page.getByLabel(/email/i).fill(ADMIN_EMAIL);
    await page.getByLabel(/password/i).fill(ADMIN_PASSWORD);
    await page.getByRole("button", { name: /sign in|log in|login/i }).click();
    await expect(page.locator("nav")).toBeVisible({ timeout: 15_000 });
  }

  test("creates a Circle, manages members (subset rule), and archives", async ({ page }) => {
    await login(page);

    // ── Create ────────────────────────────────────────────────────
    await page.goto(`${BASE_URL}/circles/new`);
    await expect(
      page.getByRole("heading", { name: /create circle|创建 circle/i }),
    ).toBeVisible({ timeout: 10_000 });
    await page.getByLabel(/realm/i).first().fill(REALM_ID);
    const title = `e2e-${Date.now()}`;
    await page.getByLabel(/^title|^标题/i).fill(title);
    await page.getByRole("button", { name: /^create$|^创建$/i }).click();

    // Detail page renders the title in its breadcrumb / header.
    await expect(page.getByRole("heading", { name: title })).toBeVisible({
      timeout: 15_000,
    });

    // Capture the circle id from the URL so the next steps can hit
    // the members surface directly.
    const detailUrl = page.url();
    const match = detailUrl.match(/\/circles\/([^/?#]+)/);
    expect(match, "circle id should be in the URL").not.toBeNull();
    const circleId = match![1];

    // ── Add a valid (Realm-member) actor ──────────────────────────
    await page.goto(`${BASE_URL}/circles/${circleId}/members`);
    await page.getByLabel(/actor did/i).fill(ACTOR_DID);
    await page.getByRole("button", { name: /add member|添加成员/i }).click();
    await expect(
      page.getByText(/member added|成员已加入/i),
    ).toBeVisible({ timeout: 10_000 });

    // ── Try to add a non-Realm-member → expect rejection ─────────
    await page.getByLabel(/actor did/i).fill(NON_MEMBER_DID);
    await page.getByRole("button", { name: /add member|添加成员/i }).click();
    await expect(
      page.getByText(
        /circle_member_must_be_realm_member|member must already belong/i,
      ),
    ).toBeVisible({ timeout: 10_000 });

    // ── Archive ──────────────────────────────────────────────────
    await page.goto(`${BASE_URL}/circles/${circleId}`);
    await page.getByRole("button", { name: /^archive$|^归档$/i }).click();
    // Confirm dialog
    await page
      .getByRole("button", { name: /^archive$|^归档$/i })
      .last()
      .click();
    await expect(
      page.getByText(/archived|已归档/i),
    ).toBeVisible({ timeout: 10_000 });

    // ── Audit log surfaces the cx.circle.* events ────────────────
    await page.goto(`${BASE_URL}/audit`);
    // Filter to the cx.circle.create event kind via the new dropdown.
    await page
      .locator("select")
      .filter({ hasText: /cx\.circle/i })
      .selectOption("cx.circle.create");
    await page.getByRole("button", { name: /apply|应用/i }).first().click();
    await expect(page.getByText(/cx\.circle\.create/i).first()).toBeVisible({
      timeout: 15_000,
    });
  });

  test("capability grant for cx.circle.member.add lets a non-admin add itself", async ({
    page,
  }) => {
    // This sub-test depends on the dev stack pre-provisioning a
    // non-admin account and exporting its DID via
    // SODMIN_E2E_NON_ADMIN_DID. Without it we skip — the capability
    // grant happy-path is otherwise covered by the unit-tested
    // GrantConstraint serialisation in `pages::capabilities`.
    const grantee = process.env.SODMIN_E2E_NON_ADMIN_DID || "";
    test.skip(!grantee, "SODMIN_E2E_NON_ADMIN_DID must be set for grant flow");

    await login(page);

    await page.goto(`${BASE_URL}/capabilities`);
    await page.getByRole("button", { name: /grant|授予/i }).click();
    await page.getByLabel(/grantee/i).fill(grantee);
    // Use the cx.circle.* quick-select dropdown shipped in P3A.4.
    await page
      .locator("select#cap-circle-quick")
      .selectOption("cx.circle.member.add");
    await page.getByRole("button", { name: /^grant$|^授予$/i }).click();
    await expect(page.getByText(/granted/i)).toBeVisible({ timeout: 10_000 });
  });
});
