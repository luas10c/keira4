export default {
  '**/*.{ts,tsx}': ['eslint --cache', 'prettier --check'],
  '**/*.rs': () => [
    'cargo fmt --all --check',
    'cargo clippy --workspace --all-targets -- -D warnings'
  ]
}
