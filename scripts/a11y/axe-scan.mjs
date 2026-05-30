#!/usr/bin/env node

import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const baseUrl = process.argv[2];

if (!baseUrl) {
  console.error("usage: node scripts/a11y/axe-scan.mjs <base-url>");
  process.exit(2);
}

const routes = ["/login", "/", "/agents", "/agents/admin", "/agents/personal", "/circles"];
const browser = await chromium.launch();
const failures = [];

try {
  const page = await browser.newPage();
  for (const route of routes) {
    const url = new URL(route, baseUrl).toString();
    await page.goto(url, { waitUntil: "domcontentloaded" });
    await page.locator("main, form, body").first().waitFor({ timeout: 15_000 });
    const results = await new AxeBuilder({ page }).analyze();
    const serious = results.violations.filter((violation) =>
      ["critical", "serious"].includes(violation.impact || ""),
    );
    if (serious.length > 0) {
      failures.push({ route, violations: serious });
    }
  }
} finally {
  await browser.close();
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`\n${failure.route}`);
    for (const violation of failure.violations) {
      console.error(`- [${violation.impact}] ${violation.id}: ${violation.help}`);
    }
  }
  process.exit(1);
}

console.log(`axe scan passed for ${routes.length} route(s)`);
