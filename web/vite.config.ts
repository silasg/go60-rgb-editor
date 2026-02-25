import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  base: '/go60-rgb-editor/',
  server: {
    fs: {
      allow: ['..'],
    },
  },
  build: {
    outDir: 'dist',
    target: 'esnext',
  },
});
