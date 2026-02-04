Changelog
=========

[0.0.9](https://github.com/casey/filepack/releases/tag/0.0.9) - 2026-02-04
--------------------------------------------------------------------------

### Added
- Add signatures subcommand ([#229](https://github.com/casey/filepack/pull/229) by [casey](https://github.com/casey))
- Add `filepack beck32m` for converting between bech32m and hex ([#181](https://github.com/casey/filepack/pull/181) by [casey](https://github.com/casey))
- Add `filepack contains` subcommand ([#175](https://github.com/casey/filepack/pull/175) by [casey](https://github.com/casey))

### Changed
- Always respect `$XDG_DATA_DIR` ([#186](https://github.com/casey/filepack/pull/186) by [casey](https://github.com/casey))
- Encode public keys, private keys, and signature as bech32m ([#177](https://github.com/casey/filepack/pull/177) by [casey](https://github.com/casey))

### Misc
- Deny unreachable-pub lint ([#234](https://github.com/casey/filepack/pull/234) by [casey](https://github.com/casey))
- Use compact fingerprint serialization ([#233](https://github.com/casey/filepack/pull/233) by [casey](https://github.com/casey))
- Make private keys begin with public key ([#232](https://github.com/casey/filepack/pull/232) by [casey](https://github.com/casey))
- Rename signature time to timestamp ([#231](https://github.com/casey/filepack/pull/231) by [casey](https://github.com/casey))
- Always verify manifest signatures on load ([#230](https://github.com/casey/filepack/pull/230) by [casey](https://github.com/casey))
- Use seconds for signature timestamp ([#228](https://github.com/casey/filepack/pull/228) by [casey](https://github.com/casey))
- Make signature time field optional ([#227](https://github.com/casey/filepack/pull/227) by [casey](https://github.com/casey))
- Sign message fingerprint ([#226](https://github.com/casey/filepack/pull/226) by [casey](https://github.com/casey))
- Include message in signature ([#225](https://github.com/casey/filepack/pull/225) by [casey](https://github.com/casey))
- Include public key in signature ([#224](https://github.com/casey/filepack/pull/224) by [casey](https://github.com/casey))
- Test bech32 decoder errors ([#223](https://github.com/casey/filepack/pull/223) by [casey](https://github.com/casey))
- Remove static GPG and SSH test files ([#222](https://github.com/casey/filepack/pull/222) by [casey](https://github.com/casey))
- Remove message subcommand ([#221](https://github.com/casey/filepack/pull/221) by [casey](https://github.com/casey))
- Remove PGP and SSH signature support ([#220](https://github.com/casey/filepack/pull/220) by [casey](https://github.com/casey))
- Generate and verify SSH and GPG signatures in integration tests ([#219](https://github.com/casey/filepack/pull/219) by [casey](https://github.com/casey))
- Bech32 prefix instead of Bech32m ([#218](https://github.com/casey/filepack/pull/218) by [casey](https://github.com/casey))
- Remove bech32m payload struct ([#217](https://github.com/casey/filepack/pull/217) by [casey](https://github.com/casey))
- Use more flexible bech32m decoder ([#216](https://github.com/casey/filepack/pull/216) by [casey](https://github.com/casey))
- Test padding errors with prefix ([#215](https://github.com/casey/filepack/pull/215) by [casey](https://github.com/casey))
- Test bech32m padding errors ([#214](https://github.com/casey/filepack/pull/214) by [casey](https://github.com/casey))
- Test signatures generated with ssh-key ([#213](https://github.com/casey/filepack/pull/213) by [casey](https://github.com/casey))
- Test overlong PGP signature suffix error ([#212](https://github.com/casey/filepack/pull/212) by [casey](https://github.com/casey))
- Add and remove multi-character bech32m prefix strings ([#211](https://github.com/casey/filepack/pull/211) by [casey](https://github.com/casey))
- Add signature hash algorithm character ([#210](https://github.com/casey/filepack/pull/210) by [casey](https://github.com/casey))
- Disable doctests ([#209](https://github.com/casey/filepack/pull/209) by [casey](https://github.com/casey))
- Convert SSH signature tests to use pre-generated key and signature ([#208](https://github.com/casey/filepack/pull/208) by [casey](https://github.com/casey))
- Update DESIGN.md ([#207](https://github.com/casey/filepack/pull/207) by [casey](https://github.com/casey))
- Resolve signature scheme reuse open question ([#206](https://github.com/casey/filepack/pull/206) by [casey](https://github.com/casey))
- Use pre-generated files for GPG signature test ([#205](https://github.com/casey/filepack/pull/205) by [casey](https://github.com/casey))
- Add message subcommand ([#204](https://github.com/casey/filepack/pull/204) by [casey](https://github.com/casey))
- Add bech32m type enum ([#203](https://github.com/casey/filepack/pull/203) by [casey](https://github.com/casey))
- Avoid allocating when formatting bech32m strings ([#202](https://github.com/casey/filepack/pull/202) by [casey](https://github.com/casey))
- Rename bech32m data to body ([#201](https://github.com/casey/filepack/pull/201) by [casey](https://github.com/casey))
- Add signature round trip and error display tests ([#200](https://github.com/casey/filepack/pull/200) by [casey](https://github.com/casey))
- Move signature payload generation into signature scheme ([#199](https://github.com/casey/filepack/pull/199) by [casey](https://github.com/casey))
- Add signature scheme type ([#198](https://github.com/casey/filepack/pull/198) by [casey](https://github.com/casey))
- Add signature scheme versioning ([#197](https://github.com/casey/filepack/pull/197) by [casey](https://github.com/casey))
- Add PGP signature scheme ([#196](https://github.com/casey/filepack/pull/196) by [casey](https://github.com/casey))
- Add SSH signature verification integration test ([#195](https://github.com/casey/filepack/pull/195) by [casey](https://github.com/casey))
- Add SSH signature scheme ([#194](https://github.com/casey/filepack/pull/194) by [casey](https://github.com/casey))
- Add signature schemes ([#193](https://github.com/casey/filepack/pull/193) by [casey](https://github.com/casey))
- Add profile recipe ([#192](https://github.com/casey/filepack/pull/192) by [casey](https://github.com/casey))
- Remove dependency on executable-path ([#191](https://github.com/casey/filepack/pull/191) by [casey](https://github.com/casey))
- Remove release commit message template ([#190](https://github.com/casey/filepack/pull/190) by [casey](https://github.com/casey))
- Forbid non-zero padding in bech32m strings ([#189](https://github.com/casey/filepack/pull/189) by [casey](https://github.com/casey))
- Use `filepack info` in data dir tests ([#188](https://github.com/casey/filepack/pull/188) by [casey](https://github.com/casey))
- Only use fingerprint type for package fingerprints ([#187](https://github.com/casey/filepack/pull/187) by [casey](https://github.com/casey))
- Use "package fingerprint" instead "manifest fingerprint" ([#185](https://github.com/casey/filepack/pull/185) by [casey](https://github.com/casey))
- Don't hash message before signing ([#184](https://github.com/casey/filepack/pull/184) by [casey](https://github.com/casey))
- Use version character `a` for bech32m strings ([#183](https://github.com/casey/filepack/pull/183) by [casey](https://github.com/casey))
- Open question: bech32m version characters ([#182](https://github.com/casey/filepack/pull/182) by [casey](https://github.com/casey))
- Use bech32m for package fingerprints ([#180](https://github.com/casey/filepack/pull/180) by [casey](https://github.com/casey))
- Resolve some open questions in DESIGN.md ([#179](https://github.com/casey/filepack/pull/179) by [casey](https://github.com/casey))
- Pretty print manifest ([#178](https://github.com/casey/filepack/pull/178) by [casey](https://github.com/casey))
- Add `--file` to `filepack contains` ([#176](https://github.com/casey/filepack/pull/176) by [casey](https://github.com/casey))
- Update DESIGN.md ([#174](https://github.com/casey/filepack/pull/174) by [casey](https://github.com/casey))
- Require lowercase hashes ([#173](https://github.com/casey/filepack/pull/173) by [casey](https://github.com/casey))
- Note that fingerprint is globally unique identifier ([#172](https://github.com/casey/filepack/pull/172) by [casey](https://github.com/casey))
- Forbid using `--manifest` with manifest in package ([#171](https://github.com/casey/filepack/pull/171) by [casey](https://github.com/casey))
- Fix release workflow manifest creation ([#170](https://github.com/casey/filepack/pull/170) by [casey](https://github.com/casey))
- Add workflows to readme ([#169](https://github.com/casey/filepack/pull/169) by [casey](https://github.com/casey))

[0.0.8](https://github.com/casey/filepack/releases/tag/0.0.8) - 2026-01-17
--------------------------------------------------------------------------

### Changed
- Move package metadata into sub-object ([#164](https://github.com/casey/filepack/pull/164) by [casey](https://github.com/casey))
- Forbid overlong components and components containing NUL ([#157](https://github.com/casey/filepack/pull/157) by [casey](https://github.com/casey))
- Allow unknown metadata fields when verifying ([#150](https://github.com/casey/filepack/pull/150) by [casey](https://github.com/casey))
- Use YAML metadata ([#148](https://github.com/casey/filepack/pull/148) by [casey](https://github.com/casey))
- Include current time in note with `filepack create/sign --time` ([#140](https://github.com/casey/filepack/pull/140) by [casey](https://github.com/casey))
- Rename keys directory to keychain ([#134](https://github.com/casey/filepack/pull/134) by [casey](https://github.com/casey))
- Create `keys` dir and private keys with correct permissions ([#123](https://github.com/casey/filepack/pull/123) by [casey](https://github.com/casey))
- Make manifest keys mandatory ([#118](https://github.com/casey/filepack/pull/118) by [casey](https://github.com/casey))

### Added
- Add homepage URLs to metadata ([#167](https://github.com/casey/filepack/pull/167) by [casey](https://github.com/casey))
- Add package creator tag ([#166](https://github.com/casey/filepack/pull/166) by [casey](https://github.com/casey))
- Add creator metadata ([#165](https://github.com/casey/filepack/pull/165) by [casey](https://github.com/casey))
- Add description fields to metadata ([#163](https://github.com/casey/filepack/pull/163) by [casey](https://github.com/casey))
- Add metadata date fields ([#162](https://github.com/casey/filepack/pull/162) by [casey](https://github.com/casey))
- Add language metadata field ([#160](https://github.com/casey/filepack/pull/160) by [casey](https://github.com/casey))
- Add file fields to metadata ([#159](https://github.com/casey/filepack/pull/159) by [casey](https://github.com/casey))
- Add packager metadata field ([#158](https://github.com/casey/filepack/pull/158) by [casey](https://github.com/casey))
- Add more lint groups ([#152](https://github.com/casey/filepack/pull/152) by [casey](https://github.com/casey))
- Notes ([#137](https://github.com/casey/filepack/pull/137) by [casey](https://github.com/casey))
- Add `filepack info` subcommand ([#132](https://github.com/casey/filepack/pull/132) by [casey](https://github.com/casey))
- Allow signing with non-default key when creating manifest ([#129](https://github.com/casey/filepack/pull/129) by [casey](https://github.com/casey))
- Allow signing with non-default key ([#128](https://github.com/casey/filepack/pull/128) by [casey](https://github.com/casey))
- Allow printing named public key ([#126](https://github.com/casey/filepack/pull/126) by [casey](https://github.com/casey))
- Allow supplying key name to `filepack keygen` ([#125](https://github.com/casey/filepack/pull/125) by [casey](https://github.com/casey))
- Allow using named keys with `filepack verify --key` ([#124](https://github.com/casey/filepack/pull/124) by [casey](https://github.com/casey))
- Check `keys` dir and private key permissions ([#121](https://github.com/casey/filepack/pull/121) by [casey](https://github.com/casey))
- Allow asserting multiple keys when verifying ([#120](https://github.com/casey/filepack/pull/120) by [casey](https://github.com/casey))
- Add `--assert` to `filepack hash` ([#117](https://github.com/casey/filepack/pull/117) by [casey](https://github.com/casey))

### Misc
- Move metadata tests into dedicated module ([#161](https://github.com/casey/filepack/pull/161) by [casey](https://github.com/casey))
- Test filepack metadata ([#156](https://github.com/casey/filepack/pull/156) by [casey](https://github.com/casey))
- Include metadata in filepack releases ([#155](https://github.com/casey/filepack/pull/155) by [casey](https://github.com/casey))
- Document metadata purpose ([#154](https://github.com/casey/filepack/pull/154) by [casey](https://github.com/casey))
- Improve path handling ([#153](https://github.com/casey/filepack/pull/153) by [casey](https://github.com/casey))
- Use snafu for lint display implementation ([#151](https://github.com/casey/filepack/pull/151) by [casey](https://github.com/casey))
- Make verify less verbose ([#149](https://github.com/casey/filepack/pull/149) by [casey](https://github.com/casey))
- Fix lint message typo ([#147](https://github.com/casey/filepack/pull/147) by [casey](https://github.com/casey))
- Skip serializing optional note fields ([#146](https://github.com/casey/filepack/pull/146) by [casey](https://github.com/casey))
- Use distinct fingerprint type ([#144](https://github.com/casey/filepack/pull/144) by [casey](https://github.com/casey))
- Consider using BLAKE3 in signatures ([#143](https://github.com/casey/filepack/pull/143) by [casey](https://github.com/casey))
- Open question: CBOR fingerprints ([#142](https://github.com/casey/filepack/pull/142) by [casey](https://github.com/casey))
- Test that duplicate note fields are rejected ([#141](https://github.com/casey/filepack/pull/141) by [casey](https://github.com/casey))
- Forbid duplicate directory entries and note signatures ([#139](https://github.com/casey/filepack/pull/139) by [casey](https://github.com/casey))
- Add single argument test method ([#138](https://github.com/casey/filepack/pull/138) by [casey](https://github.com/casey))
- Fix DESIGN.md typo ([#136](https://github.com/casey/filepack/pull/136) by [casey](https://github.com/casey))
- Delegate key generation to keychain ([#135](https://github.com/casey/filepack/pull/135) by [casey](https://github.com/casey))
- Rename Keys to Keychain ([#133](https://github.com/casey/filepack/pull/133) by [casey](https://github.com/casey))
- Restrict key names ([#131](https://github.com/casey/filepack/pull/131) by [casey](https://github.com/casey))
- Require lowercase keys and signatures ([#130](https://github.com/casey/filepack/pull/130) by [casey](https://github.com/casey))
- Be more paranoid with private keys ([#127](https://github.com/casey/filepack/pull/127) by [casey](https://github.com/casey))
- Rename `filesystem::set_mode` to `chmod` ([#122](https://github.com/casey/filepack/pull/122) by [casey](https://github.com/casey))
- Sign readme manifest ([#119](https://github.com/casey/filepack/pull/119) by [casey](https://github.com/casey))
- Use custom test assertions ([#116](https://github.com/casey/filepack/pull/116) by [casey](https://github.com/casey))
- Print number of files, bytes, and signatures verified ([#115](https://github.com/casey/filepack/pull/115) by [casey](https://github.com/casey))
- Add open questions to DESIGN.md ([#114](https://github.com/casey/filepack/pull/114) by [casey](https://github.com/casey))
- Test that extra fields in metadata are ignored ([#113](https://github.com/casey/filepack/pull/113) by [casey](https://github.com/casey))
- Add DESIGN.md ([#112](https://github.com/casey/filepack/pull/112) by [casey](https://github.com/casey))
- Expand readme ([#111](https://github.com/casey/filepack/pull/111) by [casey](https://github.com/casey))
- Add top-level doc-comment to library ([#110](https://github.com/casey/filepack/pull/110) by [casey](https://github.com/casey))
- Simplify context hasher ([#109](https://github.com/casey/filepack/pull/109) by [casey](https://github.com/casey))

[0.0.7](https://github.com/casey/filepack/releases/tag/0.0.7) - 2026-01-11
--------------------------------------------------------------------------

### Added
- Add `size` subcommand to print out manifest total file size ([#101](https://github.com/casey/filepack/pull/101) by [casey](https://github.com/casey))
- Add json and tsv formats options to `filepack files` ([#100](https://github.com/casey/filepack/pull/100) by [casey](https://github.com/casey))
- Add files subcommand to list files in manifest ([#99](https://github.com/casey/filepack/pull/99) by [casey](https://github.com/casey))
- Use hierarchical manifest format ([#90](https://github.com/casey/filepack/pull/90) by [casey](https://github.com/casey))
- Add `filepack fingerprint` command to print manifest fingerprint ([#75](https://github.com/casey/filepack/pull/75) by [casey](https://github.com/casey))

### Changed
- Allow `filepack sign` argument to be a directory or be omitted ([#69](https://github.com/casey/filepack/pull/69) by [casey](https://github.com/casey))

### Misc
- Hash fields as single-element arrays ([#107](https://github.com/casey/filepack/pull/107) by [casey](https://github.com/casey))
- Add basic context hasher tests ([#106](https://github.com/casey/filepack/pull/106) by [casey](https://github.com/casey))
- Sort file fingerprint fields ([#105](https://github.com/casey/filepack/pull/105) by [casey](https://github.com/casey))
- Domain separate signatures ([#104](https://github.com/casey/filepack/pull/104) by [casey](https://github.com/casey))
- Test that unknown fields are rejected ([#103](https://github.com/casey/filepack/pull/103) by [casey](https://github.com/casey))
- Add load with path function to get loaded manifest path ([#102](https://github.com/casey/filepack/pull/102) by [casey](https://github.com/casey))
- Rename field hasher to fingerprint hasher ([#98](https://github.com/casey/filepack/pull/98) by [casey](https://github.com/casey))
- Simplify empty directory tracking ([#97](https://github.com/casey/filepack/pull/97) by [casey](https://github.com/casey))
- Use entries iterator to calculate total manifest size ([#94](https://github.com/casey/filepack/pull/94) by [casey](https://github.com/casey))
- Remove unused trait implementations ([#93](https://github.com/casey/filepack/pull/93) by [casey](https://github.com/casey))
- Enable item order lints ([#92](https://github.com/casey/filepack/pull/92) by [casey](https://github.com/casey))
- Hash variable-length fields ([#91](https://github.com/casey/filepack/pull/91) by [casey](https://github.com/casey))
- Share manifest struct with integration tests ([#88](https://github.com/casey/filepack/pull/88) by [casey](https://github.com/casey))
- Share entry struct with integration tests ([#87](https://github.com/casey/filepack/pull/87) by [casey](https://github.com/casey))
- Improve tests ([#86](https://github.com/casey/filepack/pull/86) by [casey](https://github.com/casey))
- Remove server ([#84](https://github.com/casey/filepack/pull/84) by [casey](https://github.com/casey))
- Serve package details ([#83](https://github.com/casey/filepack/pull/83) by [terror](https://github.com/terror))
- Add `filepack server` subcommand ([#82](https://github.com/casey/filepack/pull/82) by [terror](https://github.com/terror))
- Add `filepack archive` command ([#80](https://github.com/casey/filepack/pull/80) by [terror](https://github.com/terror))
- Document `filepack verify` in readme ([#79](https://github.com/casey/filepack/pull/79) by [casey](https://github.com/casey))
- Include man page in release archives ([#77](https://github.com/casey/filepack/pull/77) by [casey](https://github.com/casey))
- Rename "root hash" to "fingerprint" ([#74](https://github.com/casey/filepack/pull/74) by [casey](https://github.com/casey))
- Avoid using `std::fs` directly ([#73](https://github.com/casey/filepack/pull/73) by [casey](https://github.com/casey))
- Open rendered page in render recipe ([#72](https://github.com/casey/filepack/pull/72) by [casey](https://github.com/casey))
- Add render subcommand ([#71](https://github.com/casey/filepack/pull/71) by [casey](https://github.com/casey))
- Use JSON to calculate root hash ([#70](https://github.com/casey/filepack/pull/70) by [casey](https://github.com/casey))
- Don't publish SHA256SUMS ([#68](https://github.com/casey/filepack/pull/68) by [casey](https://github.com/casey))

[0.0.6](https://github.com/casey/filepack/releases/tag/0.0.6) - 2024-10-09
--------------------------------------------------------------------------

### Added
- Add sign command ([#66](https://github.com/casey/filepack/pull/66) by [casey](https://github.com/casey))
- Allow missing files when verifying with `--ignore-missing` ([#61](https://github.com/casey/filepack/pull/61) by [casey](https://github.com/casey))

### Changed
- Store signatures in manifest ([#65](https://github.com/casey/filepack/pull/65) by [casey](https://github.com/casey))
- Sign and verify root hash instead of manifest hash ([#64](https://github.com/casey/filepack/pull/64) by [casey](https://github.com/casey))
- Save metadata to `metadata.json` ([#63](https://github.com/casey/filepack/pull/63) by [casey](https://github.com/casey))
- Remove download command ([#62](https://github.com/casey/filepack/pull/62) by [casey](https://github.com/casey))

[0.0.5](https://github.com/casey/filepack/releases/tag/0.0.5) - 2024-10-07
--------------------------------------------------------------------------

### Fixed
- Download tarball and zipball using web URLs ([#59](https://github.com/casey/filepack/pull/59) by [casey](https://github.com/casey))

[0.0.4](https://github.com/casey/filepack/releases/tag/0.0.4) - 2024-10-07
--------------------------------------------------------------------------

### Added
- Verify manifest hash with `--hash` ([#51](https://github.com/casey/filepack/pull/51) by [casey](https://github.com/casey))
- Add key generation, printing, signing, and verification ([#48](https://github.com/casey/filepack/pull/48) by [casey](https://github.com/casey))
- Allow including metadata in manifest ([#44](https://github.com/casey/filepack/pull/44) by [casey](https://github.com/casey))

### Changed
- Don't allow metadata template to be included in package ([#47](https://github.com/casey/filepack/pull/47) by [casey](https://github.com/casey))

### Misc
- Publish manifest with releases ([#56](https://github.com/casey/filepack/pull/56) by [casey](https://github.com/casey))
- Optimize release build ([#57](https://github.com/casey/filepack/pull/57) by [casey](https://github.com/casey))
- Include manifests in release archives ([#55](https://github.com/casey/filepack/pull/55) by [casey](https://github.com/casey))
- Add signing utility table to prior art readme section ([#54](https://github.com/casey/filepack/pull/54) by [casey](https://github.com/casey))
- Add `gpg` and `ssh-keygen` to prior art readme section ([#53](https://github.com/casey/filepack/pull/53) by [casey](https://github.com/casey))
- Add `hashdeep` and `hashdir` to alternatives section in readme ([#52](https://github.com/casey/filepack/pull/52) by [casey](https://github.com/casey))
- Use kebab-case field names ([#46](https://github.com/casey/filepack/pull/46) by [casey](https://github.com/casey))
- Allow unknown keys in manifest but not in metadata template ([#45](https://github.com/casey/filepack/pull/45) by [casey](https://github.com/casey))

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
