# {{ escape(self.name) }}

{{ DefinitionMd { schema: self.schema } }}
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
