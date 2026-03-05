export function updateConfigText(configText: string): void {
  // DOM query justified: getElementById returns generic Element, textarea needs specific type
  const textarea = document.getElementById('config-text') as HTMLTextAreaElement | null;
  if (!textarea) return;

  // Don't update if the textarea has focus (user is editing)
  if (document.activeElement === textarea) return;

  // Don't update if the value hasn't changed (avoids cursor jump)
  if (textarea.value === configText) return;

  textarea.value = configText;
}
