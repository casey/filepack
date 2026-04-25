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
