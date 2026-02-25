let lastSetValue = '';

export function updateConfigText(configText: string): void {
  const textarea = document.getElementById('config-text') as HTMLTextAreaElement | null;
  if (!textarea) return;

  // Don't update if the textarea has focus (user is editing)
  if (document.activeElement === textarea) return;

  // Don't update if the value hasn't changed (avoids cursor jump)
  if (textarea.value === configText) return;

  lastSetValue = configText;
  textarea.value = configText;
}

export function getConfigText(): string {
  const textarea = document.getElementById('config-text') as HTMLTextAreaElement | null;
  return textarea?.value ?? '';
}

export function getLastSetValue(): string {
  return lastSetValue;
}
