- [Commands](commands.md)
%% for (name, (_command, path)) in &self.commands {
  - [{{ name }}]({{ path }})
%% }
- [Languages](languages.md)
- [Routes](routes.md)
%% for (path, relative) in &self.routes {
  - [{{ escape(path) }}]({{ relative }})
%% }
- [Schemas](schemas.md)
%% for (name, path) in &self.schemas {
  - [{{ escape(name) }}]({{ path }})
%% }
- [Tagged Hex](hex.md)
%% for (prefix, path) in &self.tags {
  - [{{ prefix }}]({{ path }})
%% }
