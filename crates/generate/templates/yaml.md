YAML
====

Filepack uses YAML for hand-authored files, namely [metadata](metadata.md).

This chapter includes the schemas of all types with YAML representations.

Schemas
-------

| schema |
|--------|
%% for (name, schema) in self.yaml {
%%   if schema.has_page() {
| [{{ escape(name) }}](yaml/{{ name.to_kebab_case() }}.md) |
%%   }
%% }
