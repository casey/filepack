watch +args='test':
  cargo watch --clear --exec '{{ args }}'

ci: lint
  cargo test

lint:
  cargo clippy --all-targets -- --deny warnings
  ./bin/forbid
  cargo fmt --all -- --check

outdated:
  cargo outdated --root-deps-only

unused:
  cargo +nightly udeps

coverage:
  cargo llvm-cov --html
  open target/llvm-cov/html/index.html
