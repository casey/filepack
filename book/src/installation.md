Installation
============

`filepack` is written in [Rust](https://www.rust-lang.org/) and can be built
from source and installed from a checked-out copy of this repo with:

```sh
cargo install --locked --path .
```

Or from [crates.io](https://crates.io/crates/filepack) with:

```sh
cargo install --locked filepack
```

See [rustup.rs](https://rustup.rs/) for installation instructions for Rust.

Pre-Built Binaries
------------------

Pre-built binaries for Linux, MacOS, and Windows can be found on [the releases
page](https://github.com/casey/filepack/releases).

You can use the following command on Linux, MacOS, or Windows to download the
latest release, just replace `DEST` with the directory where you'd like to put
`filepack`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://filepack.com/install.sh | bash -s -- --to DEST
```

For example, to install `filepack` to `~/bin`:

```sh
# create ~/bin
mkdir -p ~/bin

# download and extract filepack to ~/bin/filepack
curl --proto '=https' --tlsv1.2 -sSf https://filepack.com/install.sh | bash -s -- --to ~/bin

# add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shell's initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# filepack should now be executable
filepack --help
```

Note that `install.sh` may fail on GitHub Actions or in other environments
where many machines share IP addresses. `install.sh` calls GitHub APIs in order
to determine the latest version of `filepack` to install, and those API calls
are rate-limited on a per-IP basis. To make `install.sh` more reliable in such
circumstances, pass a specific tag to install with `--tag`.
