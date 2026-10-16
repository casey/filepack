use super::*;

#[derive(FromDeriveInput)]
#[darling(
  supports(struct_named, enum_named, enum_newtype, enum_unit),
  forward_attrs(serde)
)]
pub(crate) struct Input {
  attrs: Vec<Attribute>,
  data: Data<Variant, Field>,
  generics: Generics,
  ident: Ident,
}

impl Input {
  fn parse_attributes(&self) -> Result<Attributes> {
    let mut attributes = Attributes::default();

    for attribute in &self.attrs {
      attribute
        .parse_nested_meta(|meta| attributes.insert(&meta, ContainerAttribute::parse(&meta)?))?;
    }

    Ok(attributes)
  }

  pub(crate) fn schema(&self) -> Result<TokenStream> {
    if !self.generics.params.is_empty() {
      return Err(Error::new_spanned(
        &self.generics,
        "generics are not supported",
      ));
    }

    let attributes = self.parse_attributes()?;

    let name = &self.ident;

    let schema_name = name.to_string();

    let strict = attributes.strict;

    let schema = match &self.data {
      Data::Enum(variants) => {
        if attributes.transparent {
          return Err(Error::new_spanned(
            name,
            "`#[serde(transparent)]` is only supported on structs",
          ));
        }

        if attributes.untagged {
          if attributes.tag.is_some() {
            return Err(Error::new_spanned(
              name,
              "`#[serde(tag)]` cannot be used with `#[serde(untagged)]`",
            ));
          }

          let variants = variants
            .iter()
            .map(Variant::ty)
            .collect::<Result<Vec<TokenStream>>>()?;

          quote! {
            json::Schema::UntaggedEnum {
              variants: vec![#(#variants),*],
            }
          }
        } else {
          let Some(tag) = attributes.tag else {
            return Err(Error::new_spanned(
              name,
              "enums must be tagged with `#[serde(tag = \"...\")]` or `#[serde(untagged)]`",
            ));
          };

          let variants = variants
            .iter()
            .map(|variant| variant.schema(attributes.rename_all))
            .collect::<Result<Vec<TokenStream>>>()?;

          quote! {
            json::Schema::TaggedEnum {
              strict: #strict,
              tag: #tag.into(),
              variants: vec![#(#variants),*],
            }
          }
        }
      }
      Data::Struct(fields) => {
        if attributes.tag.is_some() {
          return Err(Error::new_spanned(
            name,
            "`#[serde(tag)]` is only supported on enums",
          ));
        }

        if attributes.untagged {
          return Err(Error::new_spanned(
            name,
            "`#[serde(untagged)]` is only supported on enums",
          ));
        }

        if attributes.transparent {
          let fields = fields.iter().collect::<Vec<&Field>>();

          let [field] = fields[..] else {
            return Err(Error::new_spanned(
              name,
              "transparent structs must have exactly one field",
            ));
          };

          let inner = field.ty()?;

          quote! {
            json::Schema::Wrapper {
              inner: #inner,
            }
          }
        } else {
          let fields = fields
            .iter()
            .map(|field| field.schema(attributes.rename_all))
            .collect::<Result<Vec<TokenStream>>>()?;

          quote! {
            json::Schema::Struct {
              fields: vec![#(#fields),*],
              strict: #strict,
            }
          }
        }
      }
    };

    Ok(quote! {
      impl json::Json for #name {
        fn json_schema(registry: &mut json::Registry) -> String {
          let name = String::from(#schema_name);
          if registry.reserve(&name) {
            return name;
          }
          #schema.register(&name, registry)
        }
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn errors() {
    #[track_caller]
    fn case(input: &DeriveInput, expected: &str) {
      assert_eq!(
        Input::from_derive_input(input)
          .unwrap()
          .schema()
          .err()
          .unwrap()
          .to_string(),
        expected,
      );
    }

    case(
      &syn::parse_quote! {
        #[serde(rename = "bar")]
        struct Foo {}
      },
      "unsupported `#[serde(rename)]` attribute",
    );

    case(
      &syn::parse_quote! {
        struct Foo {
          #[serde(rename = "baz")]
          bar: String,
        }
      },
      "unsupported `#[serde(rename)]` attribute",
    );

    case(
      &syn::parse_quote! {
        struct Foo {
          #[serde(with = "bar")]
          baz: String,
        }
      },
      "unsupported `#[serde(with = \"bar\")]` attribute",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          Bar,
        }
      },
      "enums must be tagged with `#[serde(tag = \"...\")]` or `#[serde(untagged)]`",
    );

    case(
      &syn::parse_quote! {
        #[serde(tag = "bar")]
        struct Foo {}
      },
      "`#[serde(tag)]` is only supported on enums",
    );

    case(
      &syn::parse_quote! {
        #[serde(deny_unknown_fields, deny_unknown_fields)]
        struct Foo {}
      },
      "duplicate `#[serde(deny_unknown_fields)]` attribute",
    );

    case(
      &syn::parse_quote! {
        #[serde(transparent)]
        struct Foo {
          bar: String,
          baz: String,
        }
      },
      "transparent structs must have exactly one field",
    );

    case(
      &syn::parse_quote! {
        #[serde(untagged)]
        enum Foo {
          Bar { baz: String },
        }
      },
      "untagged enum variants must be newtypes",
    );

    case(
      &syn::parse_quote! {
        #[serde(untagged)]
        struct Foo {}
      },
      "`#[serde(untagged)]` is only supported on enums",
    );

    case(
      &syn::parse_quote! {
        #[serde(transparent)]
        struct Foo {
          bar: Option<String>,
        }
      },
      "transparent struct fields and untagged enum variants cannot be optional",
    );
  }
}
