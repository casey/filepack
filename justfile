watch +args='test':
  cargo watch --clear --exec '{{ args }}'

ci: lint
  cargo test --workspace

lint:
  cargo clippy --workspace --all-targets -- --deny warnings
  ./bin/forbid
  cargo fmt --all -- --check

outdated:
  cargo outdated --workspace --root-deps-only

unused:
  cargo +nightly udeps --workspace

coverage:
  cargo llvm-cov --html
  open target/llvm-cov/html/index.html

update-changelog:
  echo >> CHANGELOG.md
  git log --pretty='format:- %s' >> CHANGELOG.md

update-contributors:
  cargo run --release --package update-contributors

doc:
  cargo doc --all --open

publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  git clone git@github.com:casey/filepack.git tmp/release
  cd tmp/release
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish
  cd ../..
  rm -rf tmp/release

test-release-workflow:
  -git tag -d test-release
  -git push origin :test-release
  git tag test-release
  git push origin test-release

docs:
  open docs/index.html

shellcheck-install-script:
  shellcheck docs/install.sh

test-install-script:
  rm -f tmp/filepack
  ./docs/install.sh --to tmp

list-package:
  cargo package --list --allow-dirty

commit-release:
  git commit --edit --file release-commit-message.txt

test-progress-bar:
  #!/usr/bin/env bash
  mkdir -p tmp
  rm -f tmp/filepack.json
  head -c 1073741824 /dev/urandom > tmp/data0
  for i in {1..9}; do
    cp tmp/data0 tmp/data$i
  done
  cargo run --release create tmp
  rm tmp/data*

download-release:
  #!/usr/bin/env bash
  rm -rf tmp
  mkdir -p tmp
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  cd tmp
  cargo run download --github-release casey/filepack/$VERSION

tmp:
  rm -rf tmp
  mkdir tmp

check-error-variant-order: tmp
  cat src/error.rs | rg '^  ([A-Z].*) \{' -or '$1' > tmp/original.txt
  sort tmp/original.txt > tmp/sorted.txt
  diff tmp/{original,sorted}.txt
