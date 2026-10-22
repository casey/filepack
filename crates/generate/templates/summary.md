Summary
=======

# Getting Started
- [Installation](installation.md)
- [Shell Completion Scripts](shell-completions.md)
- [Quickstart](quickstart.md)

# Guide
- [Authoring Metadata](authoring-metadata.md)

# Background
- [Why is metadata important?](why-metadata.md)
- [Alternatives and Prior Art](alternatives.md)
- [Design](design.md)

# Reference
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
- [Routes](routes.md)
%% for (path, relative) in &self.routes {
  - [{{ escape(path) }}]({{ relative }})
%% }
- [Tagged Hex](hex.md)
%% for (prefix, path) in &self.hex {
  - [{{ prefix }}]({{ path }})
%% }
- [Unstable Features](unstable-features.md)
- [YAML](yaml.md)
%% for (name, path) in &self.yaml {
  - [{{ escape(name) }}]({{ path }})
%% }
