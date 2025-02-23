export default {
  '**/*.{ts,tsx}': ['npm run lint'],
  '**/*.rs': () => [
    'cargo fmt --all --check',
    'cargo clippy --workspace --all-targets -- -D warnings'
  ]
}
