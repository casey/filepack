use {
  self::input::Input,
  darling::{FromDeriveInput, FromField, ast::Data},
  field::Field,
  parsed_field::ParsedField,
  proc_macro::TokenStream,
  quote::quote,
  std::collections::HashSet,
  syn::{
    Attribute, DeriveInput, Error, Ident, Index, LitInt, Member, Path, Result, Type, TypePath,
  },
  usized::IntoU64,
};

mod field;
mod input;
mod parsed_field;

#[proc_macro_derive(Decode, attributes(cbor, n))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.derive_decode() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(Encode, attributes(cbor, n))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.derive_encode() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(DecodeFromStr)]
pub fn derive_decode_from_str(input: TokenStream) -> TokenStream {
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

#[proc_macro_derive(EncodeDisplay)]
pub fn derive_encode_display(input: TokenStream) -> TokenStream {
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
