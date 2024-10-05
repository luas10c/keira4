import globals from 'globals'

import js from '@eslint/js'

import ts from '@typescript-eslint/eslint-plugin'
import parser from '@typescript-eslint/parser'

import prettier from 'eslint-plugin-prettier'

import react from 'eslint-plugin-react'
import a11y from 'eslint-plugin-jsx-a11y'
import tailwindcss from 'eslint-plugin-tailwindcss'

import jest from 'eslint-plugin-jest'
import testing from 'eslint-plugin-testing-library'

/** @type{import('eslint').Linter.Config[]} */
const config = [
  {
    ignores: ['node_modules', 'dist', 'src-tauri', 'coverage']
  },
  {
    files: ['**/*.ts', '**/*.tsx'],
    ignores: ['node_modules', 'dist', 'src-tauri', 'coverage'],
    plugins: {
      '@typescript-eslint': ts,
      prettier,
      react,
      'jsx-a11y': a11y,
      tailwindcss
    },
    languageOptions: {
      parser,
      parserOptions: {
        ecmaVersion: 13,
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true
        }
      },
      globals: {
        ...globals.es2022,
        ...globals.node,
        ...globals.browser,
        React: true
      }
    },
    settings: {
      react: {
        version: 'detect'
      }
    },
    rules: {
      ...js.configs.recommended.rules,
      ...ts.configs.recommended.rules,
      ...prettier.configs.recommended.rules,
      ...react.configs.recommended.rules,
      ...react.configs['jsx-runtime'].rules,
      ...a11y.configs.recommended.rules,
      ...tailwindcss.configs.recommended.rules
    }
  },
  {
    files: ['**/*.spec.ts', '**/*.spec.tsx', '**/*.test.ts', '**/*.test.tsx'],
    plugins: {
      jest,
      'testing-library': testing,
      'react/jsx-runtime': react.configs['jsx-runtime']
    },
    languageOptions: {
      globals: {
        ...globals.jest
      }
    },
    rules: {
      ...jest.configs.recommended.rules,
      ...testing.configs['flat/react'].rules
    }
  }
]

export default config
