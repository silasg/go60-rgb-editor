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

export async function copyConfigToClipboard(): Promise<void> {
  const textarea = document.getElementById('config-text') as HTMLTextAreaElement | null;
  if (!textarea) return;

  await navigator.clipboard.writeText(textarea.value);

  // Brief visual feedback on the button
  const btn = document.getElementById('copy-config-btn');
  if (btn) {
    const original = btn.textContent;
    btn.textContent = '✅ Copied';
    setTimeout(() => { btn.textContent = original; }, 1500);
  }
}

export async function pasteConfigFromClipboard(): Promise<string | null> {
  try {
    return await navigator.clipboard.readText();
  } catch {
    // Clipboard read denied — fall back to prompt
    const text = prompt('Paste your config text:');
    return text ?? null;
  }
}
