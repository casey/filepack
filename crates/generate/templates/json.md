JSON
====

Filepack uses JSON for command input and output. This chapter includes the
schemas of all filepack types which have JSON representations, including
[Manifest](json/manifest.md), which is read by `filepack archive` and written
by `filepack manifest`.

Schemas
-------

| schema |
|--------|
%% for (name, schema) in self.json {
%%   if schema.has_page() {
| [{{ escape(name) }}](json/{{ name.to_kebab_case() }}.md) |
%%   }
%% }
