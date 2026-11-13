# {{ escape(self.name) }}

{{ DecoDefinitionMd { registry: self.registry, schema: self.schema } }}
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
