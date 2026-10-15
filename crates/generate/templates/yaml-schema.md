# {{ escape(self.name) }}

%% match self.schema {
%%   yaml::Schema::Array { element } => {
An array of {{ yaml(element) }}.
%%   }
%%   yaml::Schema::Enum { strict, tag, variants } => {
An enum with {{ Count::new(variants.len(), "variant") }}, distinguished by the
`{{ tag }}` field:

| {{ tag }} |
|------|
%%     for variant in variants {
%%       if variant.fields.is_empty() {
| `{{ variant.name }}` |
%%       } else {
| [`{{ variant.name }}`](#{{ variant.name }}) |
%%       }
%%     }
%%     if *strict {

Unknown fields are rejected.
%%     }
%%     for variant in variants {
%%       if !variant.fields.is_empty() {

## {{ variant.name }}

{{ YamlFieldsMd { fields: &variant.fields } }}
%%       }
%%     }
%%   }
%%   yaml::Schema::String => {
A string.
%%   }
%%   yaml::Schema::Struct { fields, strict } => {
A struct with {{ Count::new(fields.len(), "field") }}:

{{ YamlFieldsMd { fields } }}
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   yaml::Schema::Wrapper { inner } => {
A wrapper around {{ yaml(inner) }}.
%%   }
%% }
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
