import { test, expect } from '@playwright/test';

test('manage layers and fade delay', async ({ page }) => {
  // Arrange — load app and wait for layers to render
  await page.goto('/');
  const layerItems = page.locator('#layer-list .layer-item');
  await layerItems.first().waitFor({ state: 'visible', timeout: 10_000 });

  // Assert — initial state: layers visible, first layer active, fade value shown
  const initialCount = await layerItems.count();
  expect(initialCount).toBeGreaterThanOrEqual(1);
  await expect(layerItems.first()).toHaveClass(/active/);
  const fadeLabel = page.locator('#toolbar .fade-label');
  await expect(fadeLabel).toContainText(/Fade: \d+ms/);

  // Act — add a new layer (name must be alphanumeric/underscore only)
  page.once('dialog', (dialog) => dialog.accept('Test_Layer'));
  await page.locator('.layer-action-btn[data-action="add"]').click();

  // Assert — new layer appears in the list
  await expect(layerItems).toHaveCount(initialCount + 1);
  const newLayer = layerItems.filter({ hasText: 'Test_Layer' });
  await expect(newLayer).toHaveCount(1);

  // Act — duplicate the current layer
  const countBeforeDup = await layerItems.count();
  await page.locator('.layer-action-btn[data-action="duplicate"]').click();

  // Assert — layer count increased by 1
  await expect(layerItems).toHaveCount(countBeforeDup + 1);

  // Act — rename the active layer
  page.once('dialog', (dialog) => dialog.accept('Renamed_Layer'));
  await page.locator('.layer-action-btn[data-action="rename"]').click();

  // Assert — active layer name updated
  const activeLayer = page.locator('#layer-list .layer-item.active');
  await expect(activeLayer.locator('.layer-name')).toContainText('Renamed_Layer');

  // Act — switch to a different (non-active) layer
  const nonActiveLayer = page.locator('#layer-list .layer-item:not(.active)').first();
  const nonActiveLayerFadeText = await nonActiveLayer.locator('.layer-fade').textContent();
  await nonActiveLayer.click();

  // Assert — clicked layer is now active, fade label reflects its value
  const expectedFadeMs = nonActiveLayerFadeText!.replace('ms', '');
  await expect(fadeLabel).toContainText(expectedFadeMs);

  // Arrange — note current fade value
  const fadeBefore = await fadeLabel.textContent();
  const fadeValueBefore = parseInt(fadeBefore!.match(/\d+/)![0], 10);

  // Act — increase fade
  await page.locator('#toolbar .toolbar-btn', { hasText: '+' }).click();

  // Assert — fade increased by 5ms
  await expect(fadeLabel).toHaveText(`Fade: ${fadeValueBefore + 5}ms`);

  // Act — decrease fade
  await page.locator('#toolbar .toolbar-btn', { hasText: '−' }).click();

  // Assert — fade back to original
  await expect(fadeLabel).toHaveText(`Fade: ${fadeValueBefore}ms`);

  // Arrange — note layer count before delete
  const countBeforeDelete = await layerItems.count();

  // Act — delete a layer
  page.once('dialog', (dialog) => dialog.accept());
  await page.locator('.layer-action-btn[data-action="delete"]').click();

  // Assert — layer count decreased
  await expect(layerItems).toHaveCount(countBeforeDelete - 1);

  // Assert — config text reflects current state
  const configText = page.locator('#config-text');
  await expect(configText).not.toHaveValue('');
});
