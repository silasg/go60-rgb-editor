import { test, expect } from '@playwright/test';

test('copy button copies config text to clipboard', async ({ page, context }) => {
  // Arrange — grant clipboard permissions for headless Chromium
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  const configText = page.locator('#config-text');
  const configValue = await configText.inputValue();
  expect(configValue.length).toBeGreaterThan(0);

  // Act — click the copy button
  const copyBtn = page.locator('#copy-config-btn');
  await copyBtn.click();

  // Assert — clipboard contains the config text
  const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
  expect(clipboardText).toBe(configValue);

  // Assert — button shows feedback
  await expect(copyBtn).toHaveText('✅ Copied');

  // Assert — button reverts after delay
  await expect(copyBtn).toHaveText('📋 Copy', { timeout: 3000 });
});

test('paste button loads config from clipboard', async ({ page, context }) => {
  // Arrange — grant clipboard permissions and write a config snippet
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Arrange — grab current valid config, modify it, put it on clipboard
  const configText = page.locator('#config-text');
  const originalConfig = await configText.inputValue();
  await page.evaluate((text) => navigator.clipboard.writeText(text), originalConfig);

  // Act — change textarea to something invalid first
  await configText.fill('garbage');
  await page.waitForTimeout(600);
  const parseStatus = page.locator('#parse-status');
  await expect(parseStatus).toContainText(/error/i);

  // Act — click paste button to restore from clipboard
  const pasteBtn = page.locator('#paste-config-btn');
  await pasteBtn.click();
  await page.waitForTimeout(600);

  // Assert — valid config restored, parse error cleared
  await expect(parseStatus).toHaveText('');
  const restoredConfig = await configText.inputValue();
  expect(restoredConfig).toBe(originalConfig);
});
