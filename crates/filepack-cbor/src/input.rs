#![allow(clippy::needless_continue)]

use super::*;

#[derive(FromDeriveInput)]
#[darling(
  supports(struct_named, struct_newtype, enum_unit),
  forward_attrs(cbor, repr)
)]
pub(crate) struct Input {
  attrs: Vec<Attribute>,
  data: Data<(), Field>,
  ident: Ident,
}

impl Input {
  pub(crate) fn decode(&self) -> Result<proc_macro2::TokenStream> {
    match self.data {
      Data::Enum(_) => self.decode_enum(),
      Data::Struct(_) => {
        if self.is_transparent()? {
          self.decode_transparent()
        } else {
          self.decode_struct()
        }
      }
    }
  }

  pub(crate) fn decode_enum(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let repr = self.parse_repr()?;

    Ok(quote! {
      impl Decode for #name {
        fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
          let discriminant = decoder.integer()?;
          #repr::try_from(discriminant)
            .ok()
            .and_then(Self::from_repr)
            .context(decode_error::InvalidDiscriminant {
              discriminant,
              name: stringify!(#name),
            })
        }
      }
    })
  }

  pub(crate) fn decode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let decode = fields.iter().map(|field| {
      let ident = field.ident;
      let n = field.n;
      match (&field.decode_with, field.optional) {
        (Some(path), true) => quote! { let #ident = map.optional_key_with(#n, #path)?; },
        (Some(path), false) => quote! { let #ident = map.required_key_with(#n, #path)?; },
        (None, true) => quote! { let #ident = map.optional_key(#n)?; },
        (None, false) => quote! { let #ident = map.required_key(#n)?; },
      }
    });

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
    match self.data {
      Data::Enum(_) => self.encode_enum(),
      Data::Struct(_) => {
        if self.is_transparent()? {
          self.encode_transparent()
        } else {
          self.encode_struct()
        }
      }
    }
  }

  pub(crate) fn encode_enum(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let repr = self.parse_repr()?;

    Ok(quote! {
      impl Encode for #name {
        fn encode(&self, encoder: &mut Encoder) {
          (*self as #repr).encode(encoder);
        }
      }
    })
  }

  pub(crate) fn encode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let required = fields
      .iter()
      .filter(|field| !field.optional)
      .count()
      .into_u64();

    let optional = fields.iter().filter(|field| field.optional).map(|field| {
      let ident = field.ident;
      quote! { + u64::from(self.#ident.is_some()) }
    });

    let items = fields.iter().map(|field| {
      let ident = field.ident;
      let n = field.n;
      match (&field.encode_with, field.optional) {
        (Some(path), true) => quote! { map.optional_item_with(#n, self.#ident.as_ref(), #path); },
        (Some(path), false) => quote! { map.item_with(#n, &self.#ident, #path); },
        (None, true) => quote! { map.optional_item(#n, self.#ident.as_ref()); },
        (None, false) => quote! { map.item(#n, &self.#ident); },
      }
    });

    Ok(quote! {
      impl Encode for #name {
        fn encode(&self, encoder: &mut Encoder) {
          let length = #required #(#optional)*;
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
        "tuple struct must use `#[cbor(transparent)]` attribute to derive `Decode` or `Encode`",
      ));
    }

    let fields = data
      .into_iter()
      .map(Field::parse)
      .collect::<Result<Vec<ParsedField>>>()?;

    let mut n = HashSet::new();
    for (i, field) in fields.iter().enumerate() {
      if !n.insert(field.n) {
        return Err(Error::new_spanned(
          field.ident,
          format!("duplicate key {}", field.n),
        ));
      }

      if field.n != i.into_u64() {
        return Err(Error::new_spanned(
          field.ident,
          format!(
            "keys must be contiguous starting from 0: expected {i}, found {}",
            field.n
          ),
        ));
      }
    }

    Ok(fields)
  }

  fn parse_repr(&self) -> Result<Type> {
    let mut repr = None;

    for attribute in &self.attrs {
      if attribute.path().is_ident("repr") {
        if repr.is_some() {
          return Err(Error::new_spanned(attribute, "duplicate #[repr] attribute"));
        }
        repr = Some(attribute.parse_args::<Type>()?);
      }
    }

    repr.ok_or_else(|| Error::new_spanned(&self.ident, "missing #[repr(...)] attribute"))
  }

  fn transparent_member(&self) -> Result<Member> {
    let fields = self.data.as_ref().take_struct().unwrap();

    if fields.fields.len() != 1 {
      return Err(Error::new_spanned(
        &self.ident,
        "#[transparent] requires a struct with a single field",
      ));
    }

    Ok(match fields.fields[0].ident() {
      Some(ident) => Member::Named(ident.clone()),
      None => Member::Unnamed(Index::from(0)),
    })
  }
}
