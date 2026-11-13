YAML
====

Filepack uses YAML for hand-authored files, namely
[`metadata.yaml`](metadata.md).

This chapter includes the schemas of all types with YAML representations.

Schemas
-------

| schema | description |
|--------|-------------|
%% for (name, schema) in self.yaml {
%%   if let Some(summary) = schema.summary() {
{{ Row((
  format!("[{}](yaml/{}.md)", escape(name), name.to_kebab_case()),
  summary,
)) }}
%%   }
%% }
