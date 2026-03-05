import { test, expect } from '@playwright/test';

test('load config from text', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor();

  // Assert — default config loaded
  const keys = page.locator('.key');
  await expect(keys).not.toHaveCount(0);

  const coloredKey = keys.filter({ hasNot: page.locator('text="___"') }).first();
  await expect(coloredKey).toBeVisible();

  const configText = page.locator('#config-text');
  await expect(configText).not.toHaveValue('');

  const parseStatus = page.locator('#parse-status');
  await expect(parseStatus).toHaveText('');

  // Arrange — save valid config for later recovery
  const validConfig = await configText.inputValue();

  // Act — paste invalid config
  await configText.fill('this is not a valid config');
  await page.waitForTimeout(600);

  // Assert — parse error shown
  await expect(parseStatus).toContainText(/error/i);

  // Act — recover with valid config
  await configText.fill(validConfig);
  await page.waitForTimeout(600);

  // Assert — parse status cleared and keyboard re-rendered with colored keys
  await expect(parseStatus).toHaveText('');
  const recoveredColoredKey = keys.filter({ hasNot: page.locator('text="___"') }).first();
  await expect(recoveredColoredKey).toBeVisible();
});
