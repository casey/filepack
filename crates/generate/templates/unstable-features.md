Unstable Features
=================

Unstable features may change in backwards incompatible ways at any time.
Filepack as a whole is unstable, but unstable features are particularly
unstable.

Unstable features are enabled with the `--unstable` flag or by setting the
`FILEPACK_UNSTABLE` environment variable to `1`.

%% if self.features.is_empty() {
No unstable features.
%% } else {
| feature | description |
|---------|-------------|
%%   for (feature, description) in self.features {
{{ Row((format!("`{feature}`"), description)) }}
%%   }
%% }
