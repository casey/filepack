Summary
=======

[Introduction](introduction.md)

# Getting Started
- [Installation](installation.md)
- [Shell Completion](shell-completion.md)
- [Usage](usage.md)

# Guide
- [Why?](why.md)
- [Packages](packages.md)
- [Configuration](configuration.md)
- [Server](server.md)
- [Comparisons](comparisons.md)

# Reference
- [API](api.md)
%% for (path, page) in &self.api {
  - [{{ escape(path) }}]({{ page }})
%% }
- [Commands](commands.md)
%% for (name, (_command, path)) in &self.commands {
  - [{{ name }}]({{ path }})
%% }
- [Content Types](content-types.md)
- [Deco](deco.md)
%% for (name, path) in &self.deco {
  - [{{ escape(name) }}]({{ path }})
%% }
- [JSON](json.md)
%% for (name, path) in &self.json {
  - [{{ escape(name) }}]({{ path }})
%% }
- [Languages](languages.md)
- [Lints](lints.md)
- [Media Formats](media-formats.md)
- [Metadata](metadata.md)
- [Tagged Hex](hex.md)
%% for (tag, path) in &self.hex {
  - [{{ tag }}]({{ path }})
%% }
- [Unstable Features](unstable-features.md)
- [Web](web.md)
%% for (path, page) in &self.web {
  - [{{ escape(path) }}]({{ page }})
%% }
- [YAML](yaml.md)
%% for (name, path) in &self.yaml {
  - [{{ escape(name) }}]({{ path }})
%% }
