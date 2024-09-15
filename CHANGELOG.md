Changelog
=========

[0.0.3](https://github.com/casey/filepack/releases/tag/0.0.3) - 2024-09-15
--------------------------------------------------------------------------

### Added
- Allow overwriting manifest with `--force` ([#41](https://github.com/casey/filepack/pull/41) by [casey](https://github.com/casey))
- Add lint for junk files ([#38](https://github.com/casey/filepack/pull/38) by [casey](https://github.com/casey))
- Allow specifying path to manifest ([#35](https://github.com/casey/filepack/pull/35) by [casey](https://github.com/casey))

### Changed
- Allow portability lints by default ([#37](https://github.com/casey/filepack/pull/37) by [casey](https://github.com/casey))

### Misc
- Add color to error and mismatch messages ([#42](https://github.com/casey/filepack/pull/42) by [casey](https://github.com/casey))
- Update readme ([#36](https://github.com/casey/filepack/pull/36) by [casey](https://github.com/casey))
- Return Entry from Options::hash_file ([#34](https://github.com/casey/filepack/pull/34) by [casey](https://github.com/casey))
- Add progress bars ([#33](https://github.com/casey/filepack/pull/33) by [casey](https://github.com/casey))
- Improve missing file error messages ([#32](https://github.com/casey/filepack/pull/32) by [casey](https://github.com/casey))
- Add DESIGN.md ([#30](https://github.com/casey/filepack/pull/30) by [casey](https://github.com/casey))
- Include file sizes in manifest ([#29](https://github.com/casey/filepack/pull/29) by [casey](https://github.com/casey))
- Add `filpack hash` subcommand to hash single file ([#28](https://github.com/casey/filepack/pull/28) by [casey](https://github.com/casey))
- Install Rust toolchain in release workflow ([#27](https://github.com/casey/filepack/pull/27) by [casey](https://github.com/casey))

[0.0.2](https://github.com/casey/filepack/releases/tag/0.0.1) - 2024-09-06
--------------------------------------------------------------------------

### Added
- Default to current directory in `filepack create` and `filepack verify` ([#25](https://github.com/casey/filepack/pull/25) by [casey](https://github.com/casey))
- Add install script ([#21](https://github.com/casey/filepack/pull/21) by [casey](https://github.com/casey))
- Add `man` subcommand to print man page ([#10](https://github.com/casey/filepack/pull/10) by [casey](https://github.com/casey))
- Add `--print` flag to print manifest after verification ([#9](https://github.com/casey/filepack/pull/9) by [casey](https://github.com/casey))
- Add `--parallel` flag to read files in parallel ([#7](https://github.com/casey/filepack/pull/7) by [casey](https://github.com/casey))
- Pass `--mmap` to memory-map files for hashing ([#6](https://github.com/casey/filepack/pull/6) by [casey](https://github.com/casey))

### Misc
- Hash files last in `filepack create` ([#24](https://github.com/casey/filepack/pull/24) by [casey](https://github.com/casey))
- Add header to readme ([#23](https://github.com/casey/filepack/pull/23) by [casey](https://github.com/casey))
- Expand readme ([#22](https://github.com/casey/filepack/pull/22) by [casey](https://github.com/casey))
- Exclude files from packaged crate ([#20](https://github.com/casey/filepack/pull/20) by [casey](https://github.com/casey))
- Add alternatives and prior art section to readme ([#19](https://github.com/casey/filepack/pull/19) by [casey](https://github.com/casey))
- Change favicon to hash symbol ([#18](https://github.com/casey/filepack/pull/18) by [casey](https://github.com/casey))
- Make favicon abstract ([#17](https://github.com/casey/filepack/pull/17) by [casey](https://github.com/casey))
- Update CI workflow dependencies ([#16](https://github.com/casey/filepack/pull/16) by [casey](https://github.com/casey))
- Add slightly unhinged ouroboros favicon ([#15](https://github.com/casey/filepack/pull/15) by [casey](https://github.com/casey))
- Use braille on hover in docs ([#14](https://github.com/casey/filepack/pull/14) by [casey](https://github.com/casey))
- Add categories and keywords to Cargo.toml ([#13](https://github.com/casey/filepack/pull/13) by [casey](https://github.com/casey))
- Move site to `docs` directory to deploy to GitHub Pages ([#12](https://github.com/casey/filepack/pull/12) by [casey](https://github.com/casey))
- Add homepage ([#11](https://github.com/casey/filepack/pull/11) by [casey](https://github.com/casey))
- Add about text to `--help` output ([#8](https://github.com/casey/filepack/pull/8) by [casey](https://github.com/casey))
- Add `test-release-workflow` recipe ([#5](https://github.com/casey/filepack/pull/5) by [casey](https://github.com/casey))
- Use `echo {name}={value} >> $GITHUB_OUTPUT` in release workflow ([#4](https://github.com/casey/filepack/pull/4) by [casey](https://github.com/casey))

[0.0.1](https://github.com/casey/filepack/releases/tag/0.0.1) - 2024-09-02
--------------------------------------------------------------------------

### Added
- Add create and verify subcommands ([#2](https://github.com/casey/filepack/pull/2) by [casey](https://github.com/casey))

### Misc
- Initialize rust binary ([#1](https://github.com/casey/filepack/pull/1) by [casey](https://github.com/casey))
