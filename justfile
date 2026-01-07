watch +args='ltest':
  cargo watch --clear --exec '{{ args }}'

clippy: (watch 'lclippy --all-targets -- --deny warnings')

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

tmp:
  rm -rf tmp
  mkdir tmp

publish: tmp
  #!/usr/bin/env bash
  set -euxo pipefail
  git clone git@github.com:casey/filepack.git tmp
  cd tmp
  VERSION=`bin/version`
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish

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

test-progress-bar: tmp
  #!/usr/bin/env bash
  head -c 1073741824 /dev/urandom > tmp/data0
  for i in {1..9}; do
    cp tmp/data0 tmp/data$i
  done
  cargo run --release create tmp
  rm tmp/data*

check-error-variant-order: tmp
  cat src/error.rs | rg '^  ([A-Z].*) \{' -or '$1' > tmp/original.txt
  sort tmp/original.txt > tmp/sorted.txt
  diff tmp/{original,sorted}.txt

sign-release: tmp
  #!/usr/bin/env bash
  set -euxo pipefail
  VERSION=`bin/version`
  gh release download \
    --repo casey/filepack \
    --pattern filepack.json \
    --dir tmp \
    $VERSION
  cargo run sign tmp/filepack.json
  gh release upload \
    --clobber \
    --repo casey/filepack \
    $VERSION \
    tmp/filepack.json

verify-release: tmp
  #!/usr/bin/env bash
  set -euxo pipefail
  VERSION=`bin/version`
  gh release download \
    --repo casey/filepack \
    --pattern '*' \
    --dir tmp \
    $VERSION
  cargo run verify tmp --key 3c977ea3a31cd37f0b540f02f33eab158f2ed7449f42b05613c921181aa95b79
