import { defineConfig, globalIgnores } from 'eslint/config'

import globals from 'globals'

import js from '@eslint/js'

import ts from 'typescript-eslint'

import prettier from 'eslint-plugin-prettier'

import react from 'eslint-plugin-react'
import refresh from 'eslint-plugin-react-refresh'
import a11y from 'eslint-plugin-jsx-a11y'

import vitest from '@vitest/eslint-plugin'
import testing from 'eslint-plugin-testing-library'

export default defineConfig([
  globalIgnores(['node_modules', 'dist', 'coverage', 'target']),
  js.configs.recommended,
  ts.configs.recommended,
  {
    plugins: {
      prettier,
      react,
      'react-refresh': refresh,
      'jsx-a11y': a11y
    },
    languageOptions: {
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
        React: true,
        JSX: true
      }
    },
    settings: {
      react: {
        version: 'detect'
      }
    },
    rules: {
      ...prettier.configs.recommended.rules,
      ...react.configs.recommended.rules,
      ...refresh.configs.recommended.rules,
      ...a11y.configs.recommended.rules,
      'react-refresh/only-export-components': 'off'
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
