#![expect(clippy::needless_continue)]

use super::*;

#[derive(FromDeriveInput)]
#[darling(
  supports(struct_named, struct_newtype, enum_named, enum_unit),
  forward_attrs(cbor)
)]
pub(crate) struct Input {
  attrs: Vec<Attribute>,
  data: Data<Variant, Field>,
  generics: Generics,
  ident: Ident,
}

impl Input {
  pub(crate) fn decode(&self) -> Result<proc_macro2::TokenStream> {
    let Attributes {
      transparent,
      validate,
    } = Attributes::parse(&self.attrs)?;

    if validate && !transparent {
      return Err(Error::new_spanned(
        &self.ident,
        "#[cbor(validate)] requires #[cbor(transparent)]",
      ));
    }

    match self.data {
      Data::Enum(_) => {
        if transparent {
          Err(Error::new_spanned(
            &self.ident,
            "#[cbor(transparent)] cannot be used with enums",
          ))
        } else {
          self.decode_enum()
        }
      }
      Data::Struct(_) => {
        if transparent {
          self.decode_transparent(validate)
        } else {
          self.decode_struct()
        }
      }
    }
  }

  pub(crate) fn decode_enum(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let variants = self.parse_variants()?;

    let unit_arms = variants
      .iter()
      .filter(|variant| variant.fields.is_empty())
      .map(|ParsedVariant { ident, n, .. }| {
        quote! { #n => Ok(Self::#ident), }
      });

    let field_arms = variants
      .iter()
      .filter(|variant| !variant.fields.is_empty())
      .map(|ParsedVariant { fields, ident, n }| {
        let decode = ParsedField::decode(fields);
        let idents = fields.iter().map(|field| field.ident);
        quote! {
          #n => {
            let decoder = array.element()?;
            let mut map = decoder.map::<u64>()?;
            #(#decode)*
            map.finish()?;
            array.finish()?;
            Ok(Self::#ident {
              #(#idents,)*
            })
          }
        }
      });

    Ok(quote! {
      impl Decode for #name {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          match decoder.peek()? {
            MajorType::UnsignedInteger => {
              let discriminant = decoder.integer()?;
              match discriminant {
                #(#unit_arms)*
                _ => Err(decode_error::InvalidDiscriminant {
                  discriminant,
                  name: stringify!(#name),
                }.build()),
              }
            }
            MajorType::Array => {
              let mut array = decoder.array()?;
              let discriminant = array.item::<u64>()?;
              match discriminant {
                #(#field_arms)*
                _ => Err(decode_error::InvalidDiscriminant {
                  discriminant,
                  name: stringify!(#name),
                }.build()),
              }
            }
            actual => Err(decode_error::UnexpectedVariantType { actual }.build()),
          }
        }
      }
    })
  }

  pub(crate) fn decode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let decode = ParsedField::decode(&fields);

    let fields = fields.iter().map(|field| field.ident);

    Ok(quote! {
      impl Decode for #name {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          let mut map = decoder.map::<u64>()?;
          #(#decode)*
          map.finish()?;
          Ok(Self {
            #(#fields,)*
          })
        }
      }
    })
  }

  pub(crate) fn decode_transparent(&self, validate: bool) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let member = self.transparent_member()?;

    let constructor = match &member {
      Member::Named(ident) => quote! { Self { #ident: Decode::decode(decoder)? } },
      Member::Unnamed(_) => quote! { Self(Decode::decode(decoder)?) },
    };

    let body = if validate {
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

    let mut generics = self.generics(parse_quote!(Decode));

    if validate {
      let (_, ty_generics, _) = self.generics.split_for_impl();

      generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#name #ty_generics: Validate));
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
      impl #impl_generics Decode for #name #ty_generics #where_clause {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          #body
        }
      }
    })
  }

  pub(crate) fn encode(&self) -> Result<proc_macro2::TokenStream> {
    let Attributes { transparent, .. } = Attributes::parse(&self.attrs)?;

    match self.data {
      Data::Enum(_) => {
        if transparent {
          Err(Error::new_spanned(
            &self.ident,
            "#[cbor(transparent)] cannot be used with enums",
          ))
        } else {
          self.encode_enum()
        }
      }
      Data::Struct(_) => {
        if transparent {
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
        quote! { Self::#ident => #n.encode(encoder), }
      } else {
        let idents = fields.iter().map(|field| field.ident);
        let (length, items) = ParsedField::encode(fields, Receiver::Binding);
        quote! {
          Self::#ident { #(#idents),* } => {
            let mut array = encoder.array(2);
            array.item(#n);
            let mut map = array.element().map::<u64>(#length);
            #(#items)*
          }
        }
      }
    });

    Ok(quote! {
      impl Encode for #name {
        fn encode(&self, encoder: &mut Encoder) {
          match self {
            #(#arms)*
          }
        }
      }
    })
  }

  pub(crate) fn encode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let (length, items) = ParsedField::encode(&fields, Receiver::Field);

    Ok(quote! {
      impl Encode for #name {
        fn encode(&self, encoder: &mut Encoder) {
          let mut map = encoder.map::<u64>(#length);
          #(#items)*
        }
      }
    })
  }

  pub(crate) fn encode_transparent(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let member = self.transparent_member()?;

    let generics = self.generics(parse_quote!(Encode));

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

  fn parse_fields(&self) -> Result<Vec<ParsedField>> {
    let data = self.data.as_ref().take_struct().unwrap();

    if data.is_tuple() {
      return Err(Error::new_spanned(
        &self.ident,
        "tuple structs must use `#[cbor(transparent)]` attribute to derive `Decode` or `Encode`",
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
        "#[transparent] requires a struct with a single field",
      ));
    }

    for field in &fields.fields {
      if let Some(attr) = field.n_attribute() {
        return Err(Error::new_spanned(
          attr,
          "#[n] attribute cannot be used with #[cbor(transparent)]",
        ));
      }
    }

    Ok(match fields.fields[0].ident() {
      Some(ident) => Member::Named(ident.clone()),
      None => Member::Unnamed(Index::from(0)),
    })
  }
}
