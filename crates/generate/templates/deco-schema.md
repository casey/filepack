# {{ escape(self.name) }}

{{ DecoDefinitionMd { schema: self.schema } }}
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
