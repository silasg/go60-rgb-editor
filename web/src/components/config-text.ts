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
  // DOM query justified: getElementById returns generic Element, textarea needs specific type
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

function hasExistingConfig(): boolean {
  // DOM query justified: getElementById returns generic Element, textarea needs specific type
  const textarea = document.getElementById('config-text') as HTMLTextAreaElement | null;
  return textarea !== null && textarea.value.trim().length > 0;
}

function confirmOverwrite(): boolean {
  return confirm('This will overwrite the current config. Continue?');
}

export async function pasteConfigFromClipboard(): Promise<string | null> {
  if (hasExistingConfig() && !confirmOverwrite()) return null;

  try {
    return await navigator.clipboard.readText();
  } catch {
    // Clipboard read denied — fall back to prompt
    const text = prompt('Paste your config text:');
    return text ?? null;
  }
}

export function openConfigFile(): Promise<string | null> {
  if (hasExistingConfig() && !confirmOverwrite()) {
    return Promise.resolve(null);
  }

  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.txt,.conf,.cfg';
    input.style.display = 'none';
    document.body.appendChild(input);

    input.addEventListener('change', () => {
      const file = input.files?.[0];
      input.remove();
      if (!file) { resolve(null); return; }
      readFileAsText(file, resolve);
    });

    input.addEventListener('cancel', () => {
      input.remove();
      resolve(null);
    });

    input.click();
  });
}

function readFileAsText(
  file: File,
  resolve: (value: string | null) => void,
): void {
  const reader = new FileReader();
  reader.onload = () => {
    resolve(typeof reader.result === 'string' ? reader.result : null);
  };
  reader.onerror = () => { resolve(null); };
  reader.readAsText(file);
}
