import { test, expect } from '@playwright/test';

test('load config from text', async ({ page, context }) => {
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

  // Act — paste config from clipboard via button
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);

  // Put valid config on clipboard, break the textarea, then paste to recover
  await page.evaluate((text) => navigator.clipboard.writeText(text), validConfig);
  await configText.fill('garbage');
  await page.waitForTimeout(600);
  await expect(parseStatus).toContainText(/error/i);

  // Accept overwrite confirmation, then click paste
  page.once('dialog', (dialog) => { void dialog.accept(); });
  const pasteBtn = page.locator('#paste-config-btn');
  await pasteBtn.click();
  await page.waitForTimeout(600);

  // Assert — valid config restored from clipboard, parse error cleared
  await expect(parseStatus).toHaveText('');
  const restoredConfig = await configText.inputValue();
  expect(restoredConfig).toBe(validConfig);
});

test('paste overwrite cancelled preserves current config', async ({ page, context }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor();

  const configText = page.locator('#config-text');
  const currentConfig = await configText.inputValue();

  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.evaluate(() => navigator.clipboard.writeText('replacement config'));

  // Act — dismiss overwrite confirmation
  page.once('dialog', (dialog) => { void dialog.dismiss(); });
  await page.locator('#paste-config-btn').click();
  await page.waitForTimeout(200);

  // Assert — config unchanged
  await expect(configText).toHaveValue(currentConfig);
});

test('open config file', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor();

  const configText = page.locator('#config-text');
  const validConfig = await configText.inputValue();
  const parseStatus = page.locator('#parse-status');

  // Act — accept overwrite, then select file via file chooser
  page.once('dialog', (dialog) => { void dialog.accept(); });
  const fileChooserPromise = page.waitForEvent('filechooser');
  await page.locator('#open-config-btn').click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles({
    name: 'config.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from(validConfig),
  });
  await page.waitForTimeout(600);

  // Assert — config loaded successfully
  await expect(parseStatus).toHaveText('');
  await expect(configText).toHaveValue(validConfig);
});
