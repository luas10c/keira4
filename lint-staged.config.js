const config = {
  '**/*.tsx': [
    'eslint . --config .eslintrc.cjs --fix',
    'bash -c tsc --project tsconfig.json --noEmit --pretty'
  ]
}

export default config
