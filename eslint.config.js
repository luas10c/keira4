import { defineConfig, globalIgnores } from 'eslint/config'

import globals from 'globals'

import js from '@eslint/js'

import ts from 'typescript-eslint'

import stylistic from '@stylistic/eslint-plugin'

import react from '@eslint-react/eslint-plugin'
import a11y from 'eslint-plugin-a11y'
import tailwindcss from 'eslint-plugin-tailwindcss'

import vitest from '@vitest/eslint-plugin'
import testing from 'eslint-plugin-testing-library'

export default defineConfig([
  globalIgnores(['node_modules', 'dist', 'coverage', 'src-tauri/target']),
  js.configs.recommended,
  ts.configs.recommended,
  {
    name: 'stylistic/customized',
    ...stylistic.configs.customize({
      indent: 2,
      quotes: 'single',
      semi: false,
      commaDangle: 'never',
      jsx: true,
      arrowParens: true,
      braceStyle: '1tbs',
      blockSpacing: true,
      quoteProps: 'consistent',
      jsxQuoteStyle: 'double',
      objectCurlySpacing: 'always'
    })
  },
  react.configs.recommended,
  {
    name: 'tailwindcss/recommended',
    plugins: {
      tailwindcss
    },
    settings: {
      tailwindcss: {
        cssConfigPath: 'src/globals.css'
      }
    },
    rules: {
      ...tailwindcss.configs.recommended.rules
    }
  },
  {
    name: 'a11y/recommended',
    plugins: {
      a11y
    },
    rules: {
      ...a11y.configs.recommended.rules
    }
  },
  {
    languageOptions: {
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true
        }
      },
      globals: {
        ...globals.es2022,
        ...globals.node,
        ...globals.browser,
        React: true,
        JSX: true
      }
    }
  },
  {
    files: ['**/*.{spec,test}.ts?(x)'],
    languageOptions: {
      globals: {
        ...globals.vitest
      }
    },
    ...vitest.configs.recommended
  },
  {
    files: ['**/*.{spec,test}.ts?(x)'],
    ...testing.configs['flat/react']
  }
])
