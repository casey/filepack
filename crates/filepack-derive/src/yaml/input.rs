use super::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named, enum_named, enum_unit), forward_attrs(serde))]
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

  pub(crate) fn schema(&self) -> Result<proc_macro2::TokenStream> {
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
        let Some(tag) = attributes.tag else {
          return Err(Error::new_spanned(
            name,
            "enums must be internally tagged with `#[serde(tag = \"...\")]`",
          ));
        };

        let variants = variants
          .iter()
          .map(|variant| variant.schema(attributes.rename_all))
          .collect::<Result<Vec<proc_macro2::TokenStream>>>()?;

        quote! {
          yaml::Schema::Enum {
            strict: #strict,
            tag: #tag.into(),
            variants: vec![#(#variants),*],
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

        let fields = fields
          .iter()
          .map(|field| field.schema(attributes.rename_all))
          .collect::<Result<Vec<proc_macro2::TokenStream>>>()?;

        quote! {
          yaml::Schema::Struct {
            fields: vec![#(#fields),*],
            strict: #strict,
          }
        }
      }
    };

    Ok(quote! {
      impl yaml::Yaml for #name {
        fn yaml_schema(registry: &mut yaml::Registry) -> String {
          #schema.register(#schema_name, registry)
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
          #[serde(default)]
          bar: String,
        }
      },
      "`#[serde]` attributes are not supported on fields",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          Bar,
        }
      },
      "enums must be internally tagged with `#[serde(tag = \"...\")]`",
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
  }
}
