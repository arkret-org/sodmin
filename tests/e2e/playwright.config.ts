// Round 26 Q2 — Playwright config for the round-26+ tests/e2e/ suite.
//
// Why this is separate from the root `playwright.config.ts`: the root
// config targets the legacy compatibility example stack. The round-26
// risk-action spec assumes a different shape — it talks to a sodmin dev
// instance backed by a coauth + soland stack. Keeping them in distinct
// configs avoids cross-shaped `globalSetup` collisions while we evolve the harness.
//
// Run from the sodmin repo root:
//   npx playwright test -c tests/e2e/playwright.config.ts
//
// The spec self-skips if the required env vars are unset, so this is
// safe to invoke in a CI that hasn't yet wired the dev stack.

import { defineConfig, devices } from "@playwright/test";

const BASE_URL = process.env.SODMIN_E2E_BASE_URL || "http://localhost:8080";

export default defineConfig({
  testDir: ".",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [["list"]],
  timeout: 60_000,
  expect: { timeout: 10_000 },

  use: {
    baseURL: BASE_URL,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
