YAML
====

Filepack uses YAML for human-authored files. This chapter includes the schemas
of all filepack types which have YAML representations, including
[Metadata](yaml/metadata.md), used in `metadata.yaml`.

Schemas
-------

| schema |
|--------|
%% for name in self.yaml.keys() {
| [{{ escape(name) }}](yaml/{{ name.to_kebab_case() }}.md) |
%% }
