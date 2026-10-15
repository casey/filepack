- [Commands](commands.md)
%% for (name, (_command, path)) in &self.commands {
  - [{{ name }}]({{ path }})
%% }
- [Deco](deco.md)
%% for (name, path) in &self.deco {
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
