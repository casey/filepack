%% match self.schema {
%%   deco::Schema::Array { .. }
%%   | deco::Schema::Bytes { .. }
%%   | deco::Schema::Map { .. }
%%   | deco::Schema::NonEmptyOrderedSet { .. }
%%   | deco::Schema::SortedSet { .. } => {
%%     unreachable!()
%%   }
%%   deco::Schema::Boolean { description }
%%   | deco::Schema::SignedInteger { description }
%%   | deco::Schema::String { description }
%%   | deco::Schema::UnsignedInteger { description } => {
{{ description }}
%%   }
%%   deco::Schema::Enum { description, variants } => {
{{ description }}

An enum with {{ Count::new(variants.len(), "variant") }}:

| discriminant | variant | description |
|--------------|---------|-------------|
%%     for variant in variants {
%%       if variant.fields.is_empty() {
{{ Row((variant.n, &variant.name, &variant.description)) }}
%%       } else {
{{ Row((
  variant.n,
  format!("[{}](#{})", variant.name, variant.name.to_lowercase()),
  &variant.description,
)) }}
%%       }
%%     }
%%     for variant in variants {
%%       if !variant.fields.is_empty() {

## {{ variant.name }}

{{ DecoFieldsMd { fields: &variant.fields, registry: self.registry } }}
%%       }
%%     }
%%   }
%%   deco::Schema::Struct { description, fields, magic, strict } => {
{{ description }}

A struct with {{ Count::new(fields.len(), "field") }}:

{{ DecoFieldsMd { fields, registry: self.registry } }}
%%     if let Some(magic) = magic {

The value is encoded with magic bytes consisting of the deco encoded byte
string `filepack\0`, followed by the deco encoded byte string `{{ magic }}`,
followed by the encoded value.
%%     }
%%     if *strict {

Unknown fields are rejected.
%%     }
%%   }
%%   deco::Schema::Wrapper { description, inner } => {
{{ description }}

A wrapper around {{ self.registry.description(Chapter::Deco, inner) }}.
%%   }
%% }
