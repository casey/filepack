# {{ escape(self.name) }}

%% match self.schema {
%%   json::Schema::Array { element } => {
An array of {{ link(self.chapter, element) }}.
%%   }
%%   json::Schema::Enum { strict, tag, variants } => {
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

{{ JsonFieldsMd { chapter: self.chapter, fields: &variant.fields } }}
%%       }
%%     }
%%   }
%%   json::Schema::String => {
A string.
%%   }
%%   json::Schema::Struct { fields, strict } => {
A struct with {{ Count::new(fields.len(), "field") }}:

{{ JsonFieldsMd { chapter: self.chapter, fields } }}
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   json::Schema::Wrapper { inner } => {
A wrapper around {{ link(self.chapter, inner) }}.
%%   }
%% }
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
