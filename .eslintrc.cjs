const config = {
  root: true,
  env: {
    browser: true,
    es2023: true,
    jest: true
  },
  extends: [
    'prettier',
    'plugin:react/recommended',
    'plugin:jsx-a11y/recommended',
    'plugin:tailwindcss/recommended',
    'plugin:react-hooks/recommended',
    'plugin:react/jsx-runtime'
  ],
  plugins: ['prettier'],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true
    },
    ecmaVersion: 14
  },
  overrides: [
    {
      files: ['**/__tests__/**/*.[jt]s?(x)', '**/?(*.)+(spec|test).[jt]s?(x)'],
      extends: ['plugin:jest/recommended', 'plugin:testing-library/react'],
      plugins: ['jest']
    }
  ],
  settings: {
    react: {
      version: 'detect'
    }
  },
  rules: {
    'prettier/prettier': 'error',
    'tailwindcss/no-custom-classname': 'off'
  }
}

module.exports = config
