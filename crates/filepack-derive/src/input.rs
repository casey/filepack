use super::*;

#[derive(FromDeriveInput)]
#[darling(
  supports(struct_named, struct_newtype, enum_named, enum_unit),
  forward_attrs(deco, n)
)]
pub(crate) struct Input {
  attrs: Vec<Attribute>,
  data: Data<Variant, Field>,
  generics: Generics,
  ident: Ident,
}

impl Input {
  pub(crate) fn decode(&self) -> Result<proc_macro2::TokenStream> {
    let attributes = self.parse_attributes()?;

    match self.data {
      Data::Enum(_) => self.decode_enum(&attributes),
      Data::Struct(_) => {
        if attributes.transparent() {
          self.decode_transparent(&attributes)
        } else {
          self.decode_struct(&attributes)
        }
      }
    }
  }

  pub(crate) fn decode_enum(&self, attributes: &Attributes) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let variants = self.parse_variants()?;

    let unit_arms = variants
      .iter()
      .filter(|variant| variant.fields.is_empty())
      .map(|ParsedVariant { ident, n, .. }| {
        quote! { #n => Self::#ident, }
      });

    let field_arms = variants
      .iter()
      .filter(|variant| !variant.fields.is_empty())
      .map(|ParsedVariant { fields, ident, n }| {
        let decode = ParsedField::decode(fields);
        let idents = fields.iter().map(|field| field.ident);
        quote! {
          #n => {
            let mut map = array.decoder()?.map::<u64>()?;
            #(#decode)*
            map.finish()?;
            Self::#ident {
              #(#idents,)*
            }
          }
        }
      });

    let header = self.decode_header(attributes);

    let validate = attributes
      .validate()
      .then(|| quote! { Validate::validate(&value)?; });

    Ok(quote! {
      #header {
        fn decode(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
          let mut array = decoder.array()?;
          let discriminant = array.element::<u64>()?;
          let value = match discriminant {
            #(#unit_arms)*
            #(#field_arms)*
            _ => return Err(decode_error::InvalidDiscriminant {
              discriminant,
              name: stringify!(#name),
            }.build()),
          };
          #validate
          array.finish()?;
          Ok(value)
        }
      }
    })
  }

  fn decode_header(&self, attributes: &Attributes) -> proc_macro2::TokenStream {
    let mut header_generics = self.generics(syn::parse_quote!(Decode<'de>));

    header_generics.params.insert(0, syn::parse_quote!('de));

    let predicates = &mut header_generics.make_where_clause().predicates;

    for param in self.generics.lifetimes() {
      let lifetime = &param.lifetime;
      predicates.push(syn::parse_quote!('de: #lifetime));
    }

    if attributes.validate() {
      predicates.push(syn::parse_quote!(Self: Validate));
    }

    let name = &self.ident;

    let (impl_generics, _ty_generics, where_clause) = header_generics.split_for_impl();

    let (_impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();

    quote! {
      impl #impl_generics Decode<'de> for #name #ty_generics #where_clause
    }
  }

  pub(crate) fn decode_struct(&self, attributes: &Attributes) -> Result<proc_macro2::TokenStream> {
    let fields = self.parse_fields()?;

    let decode = ParsedField::decode(&fields);

    let fields = fields.iter().map(|field| field.ident);

    let constructor = quote! {
      Self {
        #(#fields,)*
      }
    };

    let header = self.decode_header(attributes);

    let allow_unknown_keys = attributes
      .allow_unknown_keys()
      .then(|| quote! { while let Some(_) = map.next::<&[u8]>()? {} });

    let validate = attributes
      .validate()
      .then(|| quote! { Validate::validate(&value)?; });

    Ok(quote! {
      #header {
        fn decode(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
          let mut map = decoder.map::<u64>()?;
          #(#decode)*
          #allow_unknown_keys
          map.finish()?;
          let value = #constructor;
          #validate
          Ok(value)
        }
      }
    })
  }

  pub(crate) fn decode_transparent(
    &self,
    attributes: &Attributes,
  ) -> Result<proc_macro2::TokenStream> {
    let member = self.transparent_member()?;

    let constructor = match &member {
      Member::Named(ident) => quote! { Self { #ident: Decode::decode(decoder)? } },
      Member::Unnamed(_) => quote! { Self(Decode::decode(decoder)?) },
    };

    let body = if attributes.validate() {
      quote! {
        let value = #constructor;
        Validate::validate(&value)?;
        Ok(value)
      }
    } else {
      quote! {
        Ok(#constructor)
      }
    };

    let header = self.decode_header(attributes);

    Ok(quote! {
      #header {
        fn decode(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
          #body
        }
      }
    })
  }

  pub(crate) fn encode(&self) -> Result<proc_macro2::TokenStream> {
    let attributes = self.parse_attributes()?;

    match self.data {
      Data::Enum(_) => self.encode_enum(),
      Data::Struct(_) => {
        if attributes.transparent() {
          self.encode_transparent()
        } else {
          self.encode_struct()
        }
      }
    }
  }

  pub(crate) fn encode_enum(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let variants = self.parse_variants()?;

    let arms = variants.iter().map(|ParsedVariant { fields, ident, n }| {
      if fields.is_empty() {
        quote! { Self::#ident => array.element(#n), }
      } else {
        let idents = fields.iter().map(|field| field.ident);
        let items = ParsedField::encode(fields, Receiver::Binding);
        quote! {
          Self::#ident { #(#idents),* } => {
            let mut map = array.encoder().map::<u64>();
            #(#items)*
            map.finish();
            array.element(#n);
          }
        }
      }
    });

    let generics = self.encode_generics();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
      impl #impl_generics Encode for #name #ty_generics #where_clause {
        fn encode(&self, encoder: &mut Encoder) {
          let mut array = encoder.array();
          match self {
            #(#arms)*
          }
          array.finish();
        }
      }
    })
  }

  fn encode_generics(&self) -> Generics {
    self.generics(syn::parse_quote!(Encode))
  }

  pub(crate) fn encode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let items = ParsedField::encode(&fields, Receiver::Field);

    let generics = self.encode_generics();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
      impl #impl_generics Encode for #name #ty_generics #where_clause {
        fn encode(&self, encoder: &mut Encoder) {
          let mut map = encoder.map::<u64>();
          #(#items)*
          map.finish();
        }
      }
    })
  }

  pub(crate) fn encode_transparent(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let member = self.transparent_member()?;

    let generics = self.encode_generics();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
      impl #impl_generics Encode for #name #ty_generics #where_clause {
        fn encode(&self, encoder: &mut Encoder) {
          self.#member.encode(encoder);
        }
      }
    })
  }

  fn generics(&self, bound: TypeParamBound) -> Generics {
    let mut generics = self.generics.clone();

    for param in generics.type_params_mut() {
      param.bounds.push(bound.clone());
    }

    generics
  }

  fn parse_attributes(&self) -> Result<Attributes> {
    let mut attributes = HashSet::new();

    for attribute in &self.attrs {
      if attribute.path().is_ident("n") {
        return Err(Error::new_spanned(
          attribute,
          "`#[n]` attributes can only be used on struct fields and enum variants",
        ));
      }

      if !attribute.path().is_ident("deco") {
        continue;
      }

      attribute.parse_nested_meta(|meta| {
        let attribute = meta
          .path
          .require_ident()?
          .to_string()
          .parse::<ContainerAttribute>()
          .map_err(|_| meta.error("unknown `#[deco(...)]` attribute"))?;

        if !meta.input.is_empty() && !meta.input.peek(syn::Token![,]) {
          return Err(meta.error(format!("`#[deco({attribute})]` does not take a value")));
        }

        if self.data.is_enum() {
          match attribute {
            ContainerAttribute::AllowUnknownKeys | ContainerAttribute::Transparent => {
              return Err(meta.error(format!("`#[deco({attribute})]` cannot be used with enums")));
            }
            ContainerAttribute::Validate => {}
          }
        }

        if !attributes.insert(attribute) {
          return Err(meta.error(format!("duplicate `#[deco({attribute})]` attribute")));
        }

        if attributes.contains(&ContainerAttribute::AllowUnknownKeys)
          && attributes.contains(&ContainerAttribute::Transparent)
        {
          return Err(
            meta.error("`#[deco(allow_unknown_keys)]` cannot be used with `#[deco(transparent)]`"),
          );
        }

        Ok(())
      })?;
    }

    Ok(Attributes(attributes))
  }

  fn parse_fields(&self) -> Result<Vec<ParsedField>> {
    let data = self.data.as_ref().take_struct().unwrap();

    if data.is_tuple() {
      return Err(Error::new_spanned(
        &self.ident,
        "tuple structs must use `#[deco(transparent)]` attribute to derive `Decode` or `Encode`",
      ));
    }

    let fields = data
      .into_iter()
      .map(Field::parse)
      .collect::<Result<Vec<ParsedField>>>()?;

    validate_numbers(fields.iter().map(|field| (field.ident, field.n)))?;

    Ok(fields)
  }

  fn parse_variants(&self) -> Result<Vec<ParsedVariant>> {
    let variants = self
      .data
      .as_ref()
      .take_enum()
      .unwrap()
      .into_iter()
      .map(Variant::parse)
      .collect::<Result<Vec<ParsedVariant>>>()?;

    validate_numbers(variants.iter().map(|variant| (variant.ident, variant.n)))?;

    Ok(variants)
  }

  fn transparent_member(&self) -> Result<Member> {
    let fields = self.data.as_ref().take_struct().unwrap();

    if fields.fields.len() != 1 {
      return Err(Error::new_spanned(
        &self.ident,
        "`#[deco(transparent)]` can only be used on single-field structs",
      ));
    }

    for field in &fields.fields {
      if let Some(attr) = field.n_attribute() {
        return Err(Error::new_spanned(
          attr,
          "`#[n]` attributes cannot be used with `#[deco(transparent)]`",
        ));
      }

      if let Some(attr) = field.deco_attribute() {
        return Err(Error::new_spanned(
          attr,
          "`#[deco(...)]` field attributes cannot be used with `#[deco(transparent)]`",
        ));
      }
    }

    Ok(match fields.fields[0].ident() {
      Some(ident) => Member::Named(ident.clone()),
      None => Member::Unnamed(Index::from(0)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn attribute_errors() {
    #[track_caller]
    fn case(input: &DeriveInput, expected: &str) {
      assert_eq!(
        Input::from_derive_input(input)
          .unwrap()
          .decode()
          .err()
          .unwrap()
          .to_string(),
        expected,
      );
    }

    case(
      &syn::parse_quote! {
        #[deco(validate, validate)]
        struct Foo {}
      },
      "duplicate `#[deco(validate)]` attribute",
    );

    case(
      &syn::parse_quote! {
        #[deco(allow_unknown_keys)]
        enum Foo {}
      },
      "`#[deco(allow_unknown_keys)]` cannot be used with enums",
    );

    case(
      &syn::parse_quote! {
        #[deco(allow_unknown_keys, transparent)]
        struct Foo {}
      },
      "`#[deco(allow_unknown_keys)]` cannot be used with `#[deco(transparent)]`",
    );

    case(
      &syn::parse_quote! {
        #[n(0)]
        struct Foo {}
      },
      "`#[n]` attributes can only be used on struct fields and enum variants",
    );

    case(
      &syn::parse_quote! {
        #[deco(transparent = true)]
        struct Foo {}
      },
      "`#[deco(transparent)]` does not take a value",
    );

    case(
      &syn::parse_quote! {
        #[deco(transparent)]
        struct Foo {
          bar: u64,
          baz: u64,
        }
      },
      "`#[deco(transparent)]` can only be used on single-field structs",
    );

    case(
      &syn::parse_quote! {
        #[deco(transparent)]
        struct Foo(#[deco(decode_with = bar)] u64);
      },
      "`#[deco(...)]` field attributes cannot be used with `#[deco(transparent)]`",
    );

    case(
      &syn::parse_quote! {
        enum Foo {
          #[deco(transparent)]
          Bar,
        }
      },
      "`#[deco(...)]` attributes cannot be used on enum variants",
    );
  }
}
