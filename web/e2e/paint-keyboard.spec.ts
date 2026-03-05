import { test, expect } from '@playwright/test';

test('paint and edit a keyboard layout', async ({ page }) => {
  // Arrange — load app and wait for WASM to render keyboard keys
  await page.goto('/');
  const keys = page.locator('.key');
  await keys.first().waitFor({ state: 'visible', timeout: 10_000 });

  // Assert — initial state: keyboard halves, palette, layers, config
  const leftKeys = page.locator('#keyboard-left .key');
  const rightKeys = page.locator('#keyboard-right .key');
  await expect(leftKeys).not.toHaveCount(0);
  await expect(rightKeys).not.toHaveCount(0);

  const swatches = page.locator('.swatch');
  await expect(swatches).not.toHaveCount(0);

  const layers = page.locator('#layer-list').locator('button');
  await expect(layers).not.toHaveCount(0);

  const configText = page.locator('#config-text');
  await expect(configText).not.toBeEmpty();

  // Act — pick the RED color swatch
  const redSwatch = page.locator('#palette-regular .swatch', { hasText: 'RED' });
  await redSwatch.click();

  // Assert — RED swatch is selected
  await expect(redSwatch).toHaveClass(/selected/);

  // Arrange — find an empty key and pin its identity by data attributes
  const emptyKeyLocator = keys.filter({ hasText: '___' }).first();
  const halfAttr = await emptyKeyLocator.getAttribute('data-half');
  const rowAttr = await emptyKeyLocator.getAttribute('data-row');
  const colAttr = await emptyKeyLocator.getAttribute('data-col');
  const targetKey = page.locator(`.key[data-half="${halfAttr}"][data-row="${rowAttr}"][data-col="${colAttr}"]`);
  const configBefore = await configText.inputValue();

  // Act — paint the empty key red
  await targetKey.click();

  // Assert — key turned red, cursor moved, modified indicator shown, config changed
  await expect(targetKey).toHaveCSS('background-color', 'rgb(255, 0, 0)');
  await expect(targetKey).toHaveClass(/cursor/);
  const modifiedIndicator = page.locator('#toolbar .modified-indicator');
  await expect(modifiedIndicator).toBeVisible();
  const configAfterPaint = await configText.inputValue();
  expect(configAfterPaint).not.toBe(configBefore);

  // Act — select the clear swatch ("___") and clear a colored key
  const clearSwatch = page.locator('#palette-regular .swatch').first();
  await clearSwatch.click();

  // Assert — clear swatch is selected
  await expect(clearSwatch).toHaveClass(/selected/);

  // Arrange — find a key that HAS a color (not "___") and pin its identity
  const coloredKeyLocator = keys.filter({ hasNotText: '___' }).first();
  const cHalf = await coloredKeyLocator.getAttribute('data-half');
  const cRow = await coloredKeyLocator.getAttribute('data-row');
  const cCol = await coloredKeyLocator.getAttribute('data-col');
  const coloredKey = page.locator(`.key[data-half="${cHalf}"][data-row="${cRow}"][data-col="${cCol}"]`);
  const coloredKeyTextBefore = await coloredKey.textContent();
  const configBeforeClear = await configText.inputValue();

  // Act — click the colored key to clear it
  await coloredKey.click();

  // Assert — key is cleared (no longer has its painted color)
  await expect(coloredKey).toHaveText('___');
  await expect(coloredKey).not.toHaveCSS('background-color', 'rgb(255, 0, 0)');
  const configAfterClear = await configText.inputValue();
  expect(configAfterClear).not.toBe(configBeforeClear);

  // Act — undo the clear
  await page.keyboard.press('Control+z');

  // Assert — key got its color back
  await expect(coloredKey).not.toHaveText('___');
  await expect(coloredKey).toHaveText(coloredKeyTextBefore!);

  // Act — redo the clear
  await page.keyboard.press('Control+y');

  // Assert — key is cleared again
  await expect(coloredKey).toHaveText('___');
});
