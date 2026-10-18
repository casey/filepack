# {{ escape(self.name) }}

%% match self.schema {
%%   json::Schema::Array { .. } | json::Schema::Map { .. } | json::Schema::Set { .. } => {
%%     unreachable!()
%%   }
%%   json::Schema::Boolean { description }
%%   | json::Schema::Bytes { description }
%%   | json::Schema::String { description }
%%   | json::Schema::UnsignedInteger { description } => {
{{ description }}
%%   }
%%   json::Schema::Hex { description, tag } => {
{{ description }}

A tagged hex string with tag [`{{ tag.prefix() }}`](../hex/{{ tag.prefix() }}.md).
%%   }
%%   json::Schema::StringEnum { description, values } => {
{{ description }}

A string with {{ Count::new(values.len(), "possible value") }}:

| value | description |
|-------|-------------|
%%     for value in values {
{{ Row((format!("`{}`", value.value), &value.description)) }}
%%     }
%%   }
%%   json::Schema::Struct { description, fields, strict } => {
{{ description }}

A struct with {{ Count::new(fields.len(), "field") }}:

{{ JsonFieldsMd { chapter: self.chapter, fields, registry: self.registry } }}
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   json::Schema::TaggedEnum { description, strict, tag, variants } => {
{{ description }}

An enum with {{ Count::new(variants.len(), "variant") }}, distinguished by the
`{{ tag }}` field:

| {{ tag }} | description |
|------|-------------|
%%     for variant in variants {
%%       if variant.fields.is_empty() {
{{ Row((format!("`{}`", variant.name), &variant.description)) }}
%%       } else {
{{ Row((
  format!("[`{}`](#{})", variant.name, variant.name),
  &variant.description,
)) }}
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
%%   json::Schema::UntaggedEnum { description, variants } => {
{{ description }}

An untagged enum with {{ Count::new(variants.len(), "variant") }}:

| schema |
|--------|
%%     for variant in variants {
| {{ self.registry.description(self.chapter, variant) }} |
%%     }
%%   }
%%   json::Schema::Wrapper { description, inner } => {
{{ description }}

A wrapper around {{ self.registry.description(self.chapter, inner) }}.
%%   }
%% }
%% if self.name == "Language" {

See [Languages](../languages.md) for supported language codes.
%% }
