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
  ident: Ident,
}

impl Input {
  pub(crate) fn decode(&self) -> Result<proc_macro2::TokenStream> {
    let transparent = self.is_transparent()?;

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
          self.decode_transparent()
        } else {
          self.decode_struct()
        }
      }
    }
  }

  pub(crate) fn decode_enum(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let variants = self.parse_variants()?;

    if variants.iter().all(|variant| variant.fields.is_empty()) {
      let arms = variants.iter().map(|ParsedVariant { ident, n, .. }| {
        quote! { #n => Ok(Self::#ident), }
      });

      return Ok(quote! {
        impl Decode for #name {
          fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
            let discriminant = decoder.integer()?;
            match discriminant {
              #(#arms)*
              _ => Err(decode_error::InvalidDiscriminant {
                discriminant,
                name: stringify!(#name),
              }.build()),
            }
          }
        }
      });
    }

    let unit_arms = variants
      .iter()
      .filter(|variant| variant.fields.is_empty())
      .map(|variant| {
        let ident = variant.ident;
        let n = variant.n;
        quote! { #n => Ok(Self::#ident), }
      });

    let field_arms = variants
      .iter()
      .filter(|variant| !variant.fields.is_empty())
      .map(|variant| {
        let ident = variant.ident;
        let n = variant.n;
        let decode = decode_field_items(&variant.fields);
        let idents = variant.fields.iter().map(|field| field.ident);
        quote! {
          #n => array.item_with(|decoder| {
            let mut map = decoder.map::<u64>()?;
            #(#decode)*
            map.finish()?;
            Ok(Self::#ident {
              #(#idents,)*
            })
          })?,
        }
      });

    Ok(quote! {
      impl Decode for #name {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          let head = decoder.head()?;
          match head.major_type {
            MajorType::UnsignedInteger => {
              let discriminant = head.value;
              match discriminant {
                #(#unit_arms)*
                _ => Err(decode_error::InvalidDiscriminant {
                  discriminant,
                  name: stringify!(#name),
                }.build()),
              }
            }
            MajorType::Array => {
              let mut array = ArrayDecoder::new(decoder, head.value);
              let discriminant = array.item::<u64>()?;
              let value = match discriminant {
                #(#field_arms)*
                _ => return Err(decode_error::InvalidDiscriminant {
                  discriminant,
                  name: stringify!(#name),
                }.build()),
              };
              array.finish()?;
              Ok(value)
            }
            actual => Err(decode_error::UnexpectedType {
              expected: MajorType::UnsignedInteger,
              actual,
            }.build()),
          }
        }
      }
    })
  }

  pub(crate) fn decode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let decode = decode_field_items(&fields);

    let idents = fields.iter().map(|field| field.ident);

    Ok(quote! {
      impl Decode for #name {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          let mut map = decoder.map::<u64>()?;
          #(#decode)*
          map.finish()?;
          Ok(Self {
            #(#idents,)*
          })
        }
      }
    })
  }

  pub(crate) fn decode_transparent(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let member = self.transparent_member()?;

    let constructor = match &member {
      Member::Named(ident) => quote! { Self { #ident: Decode::decode(decoder)? } },
      Member::Unnamed(_) => quote! { Self(Decode::decode(decoder)?) },
    };

    Ok(quote! {
      impl Decode for #name {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          Ok(#constructor)
        }
      }
    })
  }

  pub(crate) fn encode(&self) -> Result<proc_macro2::TokenStream> {
    let transparent = self.is_transparent()?;

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

    if variants.iter().all(|variant| variant.fields.is_empty()) {
      let arms = variants.iter().map(|ParsedVariant { ident, n, .. }| {
        quote! { Self::#ident => #n, }
      });

      return Ok(quote! {
        impl Encode for #name {
          fn encode(&self, encoder: &mut Encoder) {
            let discriminant = match self {
              #(#arms)*
            };
            discriminant.encode(encoder);
          }
        }
      });
    }

    let arms = variants.iter().map(|variant| {
      let ident = variant.ident;
      let n = variant.n;

      if variant.fields.is_empty() {
        quote! { Self::#ident => #n.encode(encoder), }
      } else {
        let idents = variant.fields.iter().map(|field| field.ident);
        let (length, items) = encode_field_items(&variant.fields, Receiver::Binding);
        quote! {
          Self::#ident { #(#idents),* } => {
            let mut array = encoder.array(2);
            array.item(#n);
            array.item_with(|encoder| {
              let length = #length;
              let mut map = encoder.map::<u64>(length);
              #(#items)*
            });
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

    let (length, items) = encode_field_items(&fields, Receiver::Field);

    Ok(quote! {
      impl Encode for #name {
        fn encode(&self, encoder: &mut Encoder) {
          let length = #length;
          let mut map = encoder.map::<u64>(length);
          #(#items)*
        }
      }
    })
  }

  pub(crate) fn encode_transparent(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let member = self.transparent_member()?;

    Ok(quote! {
      impl Encode for #name {
        fn encode(&self, encoder: &mut Encoder) {
          self.#member.encode(encoder);
        }
      }
    })
  }

  fn is_transparent(&self) -> Result<bool> {
    let mut transparent = false;

    for attribute in &self.attrs {
      if !attribute.path().is_ident("cbor") {
        continue;
      }

      attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("transparent") {
          if transparent {
            return Err(meta.error("duplicate `transparent` attribute"));
          }
          transparent = true;
          Ok(())
        } else {
          Err(meta.error("unknown cbor attribute"))
        }
      })?;
    }

    Ok(transparent)
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

    validate_ns(fields.iter().map(|field| (field.ident, field.n)))?;

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

    validate_ns(variants.iter().map(|variant| (variant.ident, variant.n)))?;

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

#[derive(Clone, Copy)]
enum Receiver {
  Binding,
  Field,
}

impl Receiver {
  fn base(self, ident: &Ident) -> proc_macro2::TokenStream {
    match self {
      Self::Binding => quote! { #ident },
      Self::Field => quote! { self.#ident },
    }
  }

  fn reference(self, ident: &Ident) -> proc_macro2::TokenStream {
    match self {
      Self::Binding => quote! { #ident },
      Self::Field => quote! { &self.#ident },
    }
  }
}

fn decode_field_items(fields: &[ParsedField]) -> Vec<proc_macro2::TokenStream> {
  fields
    .iter()
    .map(|field| {
      let ident = field.ident;
      let n = field.n;
      match (&field.decode_with, field.optional) {
        (Some(path), true) => quote! { let #ident = map.optional_key_with(#n, #path)?; },
        (Some(path), false) => quote! { let #ident = map.required_key_with(#n, #path)?; },
        (None, true) => quote! { let #ident = map.optional_key(#n)?; },
        (None, false) => quote! { let #ident = map.required_key(#n)?; },
      }
    })
    .collect()
}

fn encode_field_items(
  fields: &[ParsedField],
  receiver: Receiver,
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
  let required = fields
    .iter()
    .filter(|field| !field.optional)
    .count()
    .into_u64();

  let optional = fields.iter().filter(|field| field.optional).map(|field| {
    let base = receiver.base(field.ident);
    quote! { + u64::from(#base.is_some()) }
  });

  let length = quote! { #required #(#optional)* };

  let items = fields
    .iter()
    .map(|field| {
      let n = field.n;
      let base = receiver.base(field.ident);
      let reference = receiver.reference(field.ident);
      match (&field.encode_with, field.optional) {
        (Some(path), true) => quote! { map.optional_item_with(#n, #base.as_ref(), #path); },
        (Some(path), false) => quote! { map.item_with(#n, #reference, #path); },
        (None, true) => quote! { map.optional_item(#n, #base.as_ref()); },
        (None, false) => quote! { map.item(#n, #reference); },
      }
    })
    .collect();

  (length, items)
}
