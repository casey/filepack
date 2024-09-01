watch +args='test':
  cargo watch --clear --exec '{{ args }}'

ci: lint
  cargo test

lint:
  cargo clippy --all-targets -- --deny warnings
  ./bin/forbid
  cargo fmt --all -- --check
