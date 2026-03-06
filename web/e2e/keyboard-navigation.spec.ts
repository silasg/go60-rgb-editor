import { test, expect } from '@playwright/test';

test('keyboard editing journey: navigate, paint, copy/paste, palette picker, layers', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });
  const initialCursor = page.locator('.key.cursor');
  await expect(initialCursor).toBeVisible();

  const startCol = await initialCursor.getAttribute('data-col');
  const startHalf = await initialCursor.getAttribute('data-half');

  // Act & Assert — arrow keys move cursor
  await page.keyboard.press('ArrowRight');
  const afterRightCol = await page.locator('.key.cursor').getAttribute('data-col');
  if (startCol !== null && parseInt(startCol) < 5) {
    expect(parseInt(afterRightCol!)).toBe(parseInt(startCol) + 1);
  }

  // Act & Assert — Tab switches half
  await page.keyboard.press('Tab');
  const tabHalf = await page.locator('.key.cursor').getAttribute('data-half');
  expect(tabHalf).not.toBe(startHalf);

  // Switch back to left half for the rest of the journey
  await page.keyboard.press('Tab');

  // Act & Assert — number key paints color
  await page.keyboard.press('0');
  const paintedText = await page.locator('.key.cursor').textContent();
  expect(paintedText).not.toBe('___');

  // Act & Assert — Backspace clears color
  await page.keyboard.press('Backspace');
  const clearedText = await page.locator('.key.cursor').textContent();
  expect(clearedText).toBe('___');

  // Act & Assert — undo reverses the clear
  await page.keyboard.press('Control+z');
  const undoneText = await page.locator('.key.cursor').textContent();
  expect(undoneText).toBe(paintedText);

  // Act & Assert — copy color with 'c', move, paste with 'v'
  await page.keyboard.press('c');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('v');
  const pastedColor = await page.locator('.key.cursor').textContent();
  expect(pastedColor).toBe(paintedText);

  // Act & Assert — Enter opens palette picker
  await page.keyboard.press('Enter');
  const paletteCursor = page.locator('.swatch.palette-cursor');
  await expect(paletteCursor).toBeVisible();
  await expect(page.locator('#palette-section')).toHaveClass(/palette-active/);

  // Act & Assert — Escape cancels palette picker without painting
  const textBeforeCancel = await page.locator('.key.cursor').textContent();
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Escape');
  await expect(page.locator('.swatch.palette-cursor')).not.toBeVisible();
  await expect(page.locator('#palette-section')).not.toHaveClass(/palette-active/);
  const textAfterCancel = await page.locator('.key.cursor').textContent();
  expect(textAfterCancel).toBe(textBeforeCancel);

  // Act & Assert — Enter + navigate + Enter confirms palette selection
  await page.keyboard.press('ArrowRight'); // move to a fresh key
  await page.keyboard.press('Enter');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Enter');
  await expect(page.locator('.swatch.palette-cursor')).not.toBeVisible();
  const palettePickedText = await page.locator('.key.cursor').textContent();
  expect(palettePickedText).not.toBe('___');

  // Act & Assert — PageDown switches layer (if multiple layers exist)
  const layerCount = await page.locator('#layer-list .layer-item').count();
  if (layerCount > 1) {
    const initialLayerText = await page.locator('#layer-list .layer-item.active .layer-name').textContent();
    await page.keyboard.press('PageDown');
    const newLayerText = await page.locator('#layer-list .layer-item.active .layer-name').textContent();
    expect(newLayerText).not.toBe(initialLayerText);
  }
});

test('focus management journey: textarea isolation, help overlay', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });
  const cursorBefore = await page.locator('.key.cursor').getAttribute('data-col');

  // Act & Assert — keyboard shortcuts suppressed while textarea focused
  await page.locator('#config-text').focus();
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Escape');
  const cursorAfterTextarea = await page.locator('.key.cursor').getAttribute('data-col');
  expect(cursorAfterTextarea).toBe(cursorBefore);

  // Act & Assert — arrow keys work after Escape exits textarea
  await page.keyboard.press('ArrowRight');
  const cursorAfterNav = await page.locator('.key.cursor').getAttribute('data-col');
  if (cursorBefore !== null && parseInt(cursorBefore) < 5) {
    expect(parseInt(cursorAfterNav!)).toBe(parseInt(cursorBefore) + 1);
  }

  // Act & Assert — ? opens help overlay
  await page.keyboard.press('?');
  const overlay = page.locator('#help-overlay');
  await expect(overlay).not.toHaveClass(/hidden/);
  await expect(overlay.locator('h2')).toHaveText('Keyboard Shortcuts');

  // Act & Assert — ? closes help overlay
  await page.keyboard.press('?');
  await expect(overlay).toHaveClass(/hidden/);

  // Act & Assert — Escape also closes help overlay
  await page.keyboard.press('?');
  await expect(overlay).not.toHaveClass(/hidden/);
  await page.keyboard.press('Escape');
  await expect(overlay).toHaveClass(/hidden/);

  // Act & Assert — close button also closes help overlay
  await page.keyboard.press('?');
  await expect(overlay).not.toHaveClass(/hidden/);
  await page.locator('#help-close').click();
  await expect(overlay).toHaveClass(/hidden/);
});
