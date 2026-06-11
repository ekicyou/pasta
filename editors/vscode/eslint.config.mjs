// ESLint flat config for the Pasta VSCode extension (`npm run lint` = `eslint src`).
//
// Dev-only tooling (review-improvement-loop cell 3.56 / G2): lints the
// TypeScript sources with the typescript-eslint recommended rule set.
// Build outputs and vendored artifacts are ignored.
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: ['out/**', 'node_modules/**', 'wasm/**', '*.vsix'],
  },
  ...tseslint.configs.recommended,
  {
    files: ['src/**/*.ts'],
    rules: {
      // VSCode provider interfaces force unused parameters (e.g. the
      // CancellationToken); the codebase marks them with a `_` prefix.
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
        },
      ],
    },
  },
  {
    // Test files: the vscode mock and the inline test harnesses use `any`
    // for duck-typed mock plumbing; keep the rest of the recommended set.
    files: ['src/test/**/*.ts'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
    },
  },
);
