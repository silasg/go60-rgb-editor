import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist/**', '**/*.d.ts'] },

  // ── Source code: strict TypeScript + custom rules ──────────────
  {
    files: ['src/**/*.ts'],
    extends: [
      eslint.configs.recommended,
      ...tseslint.configs.strictTypeChecked,
      ...tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      // ── TypeScript skill: use both type and interface ───────────
      // "type for data, interface for behavior" — disable the rule
      // that forces one or the other.
      '@typescript-eslint/consistent-type-definitions': 'off',

      // ── TypeScript skill: explicit return types ────────────────
      '@typescript-eslint/explicit-function-return-type': ['error', {
        allowExpressions: true,
        allowTypedFunctionExpressions: true,
        allowHigherOrderFunctions: true,
      }],

      // ── TypeScript skill: prefer readonly class fields ─────────
      '@typescript-eslint/prefer-readonly': 'error',

      // ── TypeScript skill: minimize type assertions ─────────────
      // Allow 'as' (with justifying comment), forbid on object literals.
      '@typescript-eslint/consistent-type-assertions': ['error', {
        assertionStyle: 'as',
        objectLiteralTypeAssertions: 'never',
      }],

      // ── TypeScript skill: consistent type imports ──────────────
      '@typescript-eslint/consistent-type-imports': ['error', {
        prefer: 'type-imports',
        fixStyle: 'inline-type-imports',
      }],

      // ── TypeScript skill: exhaustive switches ──────────────────
      '@typescript-eslint/switch-exhaustiveness-check': 'error',

      // ── Allow numbers/booleans in template literals ────────────
      '@typescript-eslint/restrict-template-expressions': ['error', {
        allowNumber: true,
        allowBoolean: true,
      }],

      // ── Rust arch rule: function length limit (60 lines) ──────
      'max-lines-per-function': ['error', {
        max: 60,
        skipBlankLines: true,
        skipComments: true,
      }],

      // ── Coding-style skill: early returns, low nesting ────────
      'max-depth': ['error', 4],
      complexity: ['error', 15],

      // ── Coding-style skill: immutability defaults ──────────────
      'prefer-const': 'error',
      'no-var': 'error',

      // ── General strictness ─────────────────────────────────────
      eqeqeq: ['error', 'always'],
      curly: ['error', 'multi-line'],
      'no-console': 'warn',
    },
  },

  // ── Architecture: Components must not import editor-bridge ─────
  // Mirrors Rust rule: ui_no_io_access
  {
    files: ['src/components/**/*.ts'],
    rules: {
      'no-restricted-imports': ['error', {
        patterns: [{
          group: ['../editor-bridge*'],
          message: 'Architecture: components (UI) must not import editor-bridge (IO). Mirrors Rust ui_no_io_access.',
        }],
      }],
    },
  },

  // ── Architecture: editor-bridge must not import components ─────
  // Mirrors Rust rule: io_no_ui_dependency
  {
    files: ['src/editor-bridge.ts'],
    rules: {
      'no-restricted-imports': ['error', {
        patterns: [{
          group: ['./components/*'],
          message: 'Architecture: editor-bridge (IO) must not import components (UI). Mirrors Rust io_no_ui_dependency.',
        }],
      }],
    },
  },
);
