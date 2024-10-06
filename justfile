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

create-release-manifest:
  #!/usr/bin/env bash
  rm -rf tmp
  mkdir -p tmp
  cargo run create --github-release casey/filepack/0.0.3 tmp

download-release:
  #!/usr/bin/env bash
  rm -rf tmp
  mkdir -p tmp
  cd tmp
  curl -L \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    https://api.github.com/repos/casey/filepack/releases/tags/0.0.3
    # | jq -c '.assets[]' | while read asset;
  # do
    # URL=`echo "$asset" | jq -r .browser_download_url`
    # echo "Downloading $URL..."
    # curl --verbose --remote-name --location "$URL"
  # done
  # curl --verbose --remote-name --location "https://github.com/casey/filepack/archive/refs/tags/0.0.3.zip"
  # curl --verbose --remote-name --location "https://github.com/casey/filepack/archive/refs/tags/0.0.3.tar.gz"
