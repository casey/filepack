# {{ escape(self.name) }}

%% match self.schema {
%%   json::Schema::Array { .. } | json::Schema::Map { .. } | json::Schema::Set { .. } => {
%%     unreachable!()
%%   }
%%   json::Schema::Boolean => {
A boolean.
%%   }
%%   json::Schema::Bytes => {
A byte string, encoded as hexadecimal.
%%   }
%%   json::Schema::Hex { tag } => {
A tagged hex string with tag [`{{ tag.prefix() }}`](../hex/{{ tag.prefix() }}.md).
%%   }
%%   json::Schema::String => {
A string.
%%   }
%%   json::Schema::StringEnum { values } => {
A string with {{ Count::new(values.len(), "possible value") }}:

| value |
|-------|
%%     for value in values {
| `{{ value }}` |
%%     }
%%   }
%%   json::Schema::Struct { fields, strict } => {
A struct with {{ Count::new(fields.len(), "field") }}:

{{ JsonFieldsMd { chapter: self.chapter, fields, registry: self.registry } }}
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   json::Schema::TaggedEnum { strict, tag, variants } => {
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

{{ JsonFieldsMd { chapter: self.chapter, fields: &variant.fields, registry: self.registry } }}
%%       }
%%     }
%%   }
%%   json::Schema::UnsignedInteger => {
An unsigned 64-bit integer.
%%   }
%%   json::Schema::UntaggedEnum { variants } => {
An untagged enum with {{ Count::new(variants.len(), "variant") }}:

| schema |
|--------|
%%     for variant in variants {
| {{ self.registry.description(self.chapter, variant) }} |
%%     }
%%   }
%%   json::Schema::Wrapper { inner } => {
A wrapper around {{ self.registry.description(self.chapter, inner) }}.
%%   }
%% }
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
