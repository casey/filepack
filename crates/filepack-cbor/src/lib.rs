use {
  self::{
    attributes::Attributes, field::Field, input::Input, parsed_field::ParsedField,
    parsed_variant::ParsedVariant, receiver::Receiver, variant::Variant,
  },
  darling::{FromDeriveInput, FromField, FromVariant, ast::Data, ast::Fields},
  proc_macro::TokenStream,
  quote::quote,
  std::collections::HashSet,
  syn::{
    Attribute, DeriveInput, Error, Ident, Index, LitInt, Member, Path, Result, Type, TypePath,
  },
  usized::IntoU64,
};

mod attributes;
mod field;
mod input;
mod parsed_field;
mod parsed_variant;
mod receiver;
mod variant;

#[proc_macro_derive(Decode, attributes(cbor, n))]
pub fn decode(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.decode() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(DecodeFromStr)]
pub fn decode_from_str(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  quote! {
    impl Decode for #name {
      fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder
          .text()?
          .parse::<Self>()
          .map_err(|source| DecodeError::FromStr {
            name: stringify!(#name),
            source: Box::new(source),
          })
      }
    }
  }
  .into()
}

#[proc_macro_derive(Encode, attributes(cbor, n))]
pub fn encode(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.encode() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(EncodeDisplay)]
pub fn encode_display(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  quote! {
    impl Encode for #name {
      fn encode(&self, encoder: &mut Encoder) {
        self.to_string().encode(encoder);
      }
    }
  }
  .into()
}

fn number(ident: &Ident, attributes: &[Attribute]) -> Result<u64> {
  let mut n = None;

  for attribute in attributes {
    if attribute.path().is_ident("n") {
      if n.is_some() {
        return Err(Error::new_spanned(attribute, "duplicate #[n] attribute"));
      }
      n = Some(attribute.parse_args::<LitInt>()?.base10_parse::<u64>()?);
    }
  }

  n.ok_or_else(|| Error::new_spanned(ident, "missing #[n(N)] attribute"))
}

fn validate_numbers<'a>(ns: impl IntoIterator<Item = (&'a Ident, u64)>) -> Result<()> {
  let mut seen = HashSet::new();

  for (i, (ident, n)) in ns.into_iter().enumerate() {
    if !seen.insert(n) {
      return Err(Error::new_spanned(
        ident,
        format!("duplicate #[n] attribute {n}"),
      ));
    }

    if n != i.into_u64() {
      return Err(Error::new_spanned(
        ident,
        format!("#[n] attributes must be contiguous starting from 0: expected {i}, found {n}"),
      ));
    }
  }

  Ok(())
}
