#![allow(clippy::needless_continue)]

use super::*;

#[derive(FromDeriveInput)]
#[darling(
  supports(struct_named, struct_newtype, enum_unit),
  forward_attrs(repr, transparent)
)]
pub(crate) struct Input {
  attrs: Vec<Attribute>,
  data: Data<(), Field>,
  ident: Ident,
}

impl Input {
  pub(crate) fn derive_decode(&self) -> Result<proc_macro2::TokenStream> {
    match self.data {
      Data::Enum(_) => self.derive_decode_enum(),
      Data::Struct(_) => {
        if self.is_transparent() {
          self.derive_decode_transparent()
        } else {
          self.derive_decode_struct()
        }
      }
    }
  }

  pub(crate) fn derive_decode_enum(&self) -> Result<proc_macro2::TokenStream> {
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

  pub(crate) fn derive_decode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    let fields = self.parse_fields()?;

    let decode = fields.iter().map(|f| {
      let ident = f.ident;
      let n = f.n;
      if f.optional {
        quote! { let #ident = map.optional_key(#n)?; }
      } else {
        quote! { let #ident = map.required_key(#n)?; }
      }
    });

    let fields = fields.iter().map(|f| f.ident);

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

  pub(crate) fn derive_decode_transparent(&self) -> Result<proc_macro2::TokenStream> {
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

  pub(crate) fn derive_encode(&self) -> Result<proc_macro2::TokenStream> {
    match self.data {
      Data::Enum(_) => self.derive_encode_enum(),
      Data::Struct(_) => {
        if self.is_transparent() {
          self.derive_encode_transparent()
        } else {
          self.derive_encode_struct()
        }
      }
    }
  }

  pub(crate) fn derive_encode_enum(&self) -> Result<proc_macro2::TokenStream> {
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

  pub(crate) fn derive_encode_struct(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;

    if self.is_transparent() {
      let member = self.transparent_member()?;
      return Ok(quote! {
        impl Encode for #name {
          fn encode(&self, encoder: &mut Encoder) {
            self.#member.encode(encoder);
          }
        }
      });
    }

    let fields = self.parse_fields()?;

    let required = fields.iter().filter(|f| !f.optional).count().into_u64();

    let optional = fields.iter().filter(|f| f.optional).map(|f| {
      let ident = f.ident;
      quote! { + u64::from(self.#ident.is_some()) }
    });

    let items = fields.iter().map(|f| {
      let ident = f.ident;
      let n = f.n;
      if f.optional {
        quote! { map.optional_item(#n, self.#ident.as_ref()); }
      } else {
        quote! { map.item(#n, &self.#ident); }
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

  pub(crate) fn derive_encode_transparent(&self) -> Result<proc_macro2::TokenStream> {
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

  fn is_transparent(&self) -> bool {
    self
      .attrs
      .iter()
      .any(|attribute| attribute.path().is_ident("transparent"))
  }

  fn parse_fields(&self) -> Result<Vec<ParsedField>> {
    let data = self.data.as_ref().take_struct().unwrap();

    if data.is_tuple() {
      return Err(Error::new_spanned(
        &self.ident,
        "tuple struct must use `#[transparent]` attribute to derive `Decode` or `Encode`",
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
