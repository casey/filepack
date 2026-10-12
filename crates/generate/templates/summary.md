- [Commands](commands.md)
%% for (name, path) in &self.commands {
  - [{{ name }}]({{ path }})
%% }
- [Routes](routes.md)
- [Schemas](schemas.md)
%% for (name, path) in &self.schemas {
  - [{{ escape(name) }}]({{ path }})
%% }
- [Tagged Hex](hex.md)
