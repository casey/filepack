use super::*;

#[derive(FromDeriveInput)]
#[darling(
  supports(struct_named, enum_named, enum_newtype, enum_unit),
  forward_attrs(doc, serde)
)]
pub(crate) struct Input {
  attrs: Vec<Attribute>,
  data: Data<Variant, Field>,
  generics: Generics,
  ident: Ident,
}

impl Input {
  fn description(&self) -> Result<Description> {
    Description::new(&self.ident, &self.attrs, "type", false, false)
  }

  fn parse_attributes(&self) -> Result<Attributes> {
    let mut attributes = Attributes::default();

    for attribute in &self.attrs {
      if !attribute.path().is_ident("serde") {
        continue;
      }

      attribute
        .parse_nested_meta(|meta| attributes.insert(&meta, ContainerAttribute::parse(&meta)?))?;
    }

    Ok(attributes)
  }

  pub(crate) fn schema(&self) -> Result<TokenStream> {
    let mut generics = self.generics.clone();

    let mut parameters = Vec::new();

    for param in &mut generics.params {
      match param {
        GenericParam::Type(param) => {
          param.bounds.push(syn::parse_quote!(json::Json));
          let ident = &param.ident;
          parameters.push(quote! { <#ident>::json_schema(registry) });
        }
        GenericParam::Const(..) => {
          return Err(Error::new_spanned(
            param,
            "const parameters are not supported",
          ));
        }
        GenericParam::Lifetime(..) => {
          return Err(Error::new_spanned(
            param,
            "lifetime parameters are not supported",
          ));
        }
      }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let attributes = self.parse_attributes()?;

    let name = &self.ident;

    let schema_name = if parameters.is_empty() {
      let name = name.to_string();
      quote! { String::from(#name) }
    } else {
      let format = format!("{name}<{}>", vec!["{}"; parameters.len()].join(", "));
      quote! { format!(#format, #(#parameters),*) }
    };

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

          let description = self.description()?;

          quote! {
            json::Schema::UntaggedEnum {
              description: #description.into(),
              variants: vec![#(#variants),*],
            }
          }
        } else if variants.iter().all(Variant::is_unit) {
          let values = variants
            .iter()
            .map(|variant| variant.value(attributes.rename_all))
            .collect::<Result<Vec<TokenStream>>>()?;

          let description = self.description()?;

          quote! {
            json::Schema::StringEnum {
              description: #description.into(),
              values: vec![#(#values),*],
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

          let description = self.description()?;

          quote! {
            json::Schema::TaggedEnum {
              description: #description.into(),
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

          let description = self.description()?;

          quote! {
            json::Schema::Wrapper {
              description: #description.into(),
              inner: #inner,
            }
          }
        } else {
          let fields = fields
            .iter()
            .map(|field| field.schema(attributes.rename_all))
            .collect::<Result<Vec<TokenStream>>>()?;

          let description = self.description()?;

          quote! {
            json::Schema::Struct {
              description: #description.into(),
              fields: vec![#(#fields),*],
              strict: #strict,
            }
          }
        }
      }
    };

    Ok(quote! {
      impl #impl_generics json::Json for #name #ty_generics #where_clause {
        fn json_schema(registry: &mut json::Registry) -> String {
          let name = #schema_name;
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
        struct Foo {
          #[json(bar)]
          baz: String,
        }
      },
      "unsupported `#[json(bar)]` attribute",
    );

    case(
      &syn::parse_quote! {
        struct Foo {
          bar: String,
        }
      },
      "fields must have doc comments",
    );

    case(
      &syn::parse_quote! {
        #[serde(tag = "type")]
        enum Foo {
          Bar {},
        }
      },
      "variants must have doc comments",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          Bar,
        }
      },
      "variants must have doc comments",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          /// Bar.
          Bar,
        }
      },
      "types must have doc comments",
    );

    case(
      &syn::parse_quote! {
        /// Foo
        enum Foo {
          /// Bar.
          Bar,
        }
      },
      "description `Foo` does not end with a period",
    );

    case(
      &syn::parse_quote! {
        struct Foo {
          #[doc = include_str!("bar")]
          baz: String,
        }
      },
      "`#[doc]` attribute value must be a string literal",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          Bar { baz: String },
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
        struct Foo<const N: usize> {
          bar: [String; N],
        }
      },
      "const parameters are not supported",
    );

    case(
      &syn::parse_quote! {
        struct Foo<'a> {
          bar: &'a str,
        }
      },
      "lifetime parameters are not supported",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          #[serde(skip)]
          Bar,
        }
      },
      "unsupported `#[serde(skip)]` attribute",
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

    case(
      &syn::parse_quote! {
        struct Foo {
          bar: Option<String>,
        }
      },
      "`Option` fields must have `#[serde(skip_serializing_if)]` or the type must have `#[skip_serializing_none]`",
    );
  }
}
