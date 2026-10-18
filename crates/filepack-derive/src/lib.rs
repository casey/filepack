use {
  self::{
    functions::{
      description, display_name, doc_line, number, option, path_value, type_argument,
      validate_numbers,
    },
    route_attribute::RouteAttribute,
  },
  darling::{FromDeriveInput, FromField, FromVariant, ast::Data, ast::Fields},
  heck::{ToKebabCase, ToSnakeCase},
  proc_macro2::TokenStream,
  quote::{format_ident, quote},
  std::{collections::HashSet, mem},
  strum::{EnumString, IntoStaticStr},
  syn::{
    Attribute, DeriveInput, Error, Expr, ExprLit, FnArg, GenericArgument, GenericParam, Generics,
    Ident, ItemFn, Lit, LitInt, LitStr, Path, PathArguments, ReturnType, Token, Type,
    TypeParamBound, TypePath,
    meta::ParseNestedMeta,
    parse::{Parse, ParseBuffer, ParseStream},
  },
  usized::IntoU64,
};

mod deco;
mod functions;
mod json;
mod route_attribute;

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[proc_macro_derive(Decode, attributes(deco, n))]
pub fn decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match deco::Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.decode() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(DecodeFromStr)]
pub fn decode_from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  let display_name = display_name(name);

  quote! {
    impl Decode<'_> for #name {
      fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        decoder
          .text()?
          .parse::<Self>()
          .map_err(|source| MalformedError::FromStr {
            name: #display_name,
            source: Box::new(source),
          }.into())
      }
    }
  }
  .into()
}

#[proc_macro_derive(Encode, attributes(deco, n))]
pub fn encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match deco::Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.encode() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(EncodeDisplay)]
pub fn encode_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  let schema_name = name.to_string();

  quote! {
    impl Encode for #name {
      fn deco_schema(registry: &mut deco::Registry) -> String {
        deco::Schema::Wrapper {
          inner: String::deco_schema(registry),
        }
        .register(#schema_name, registry)
      }

      fn encode(&self, encoder: &mut Encoder) {
        self.to_string().encode(encoder);
      }
    }
  }
  .into()
}

#[proc_macro_derive(Json, attributes(json))]
pub fn json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match json::Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.schema() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(Magic, attributes(deco, n))]
pub fn magic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match deco::Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.magic() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_attribute]
pub fn route(
  attribute: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let RouteAttribute { method, path } = syn::parse_macro_input!(attribute as RouteAttribute);

  let item = syn::parse_macro_input!(item as ItemFn);

  let name = &item.sig.ident;

  let route = format_ident!("{name}_route");

  let mut types = Vec::new();

  for input in &item.sig.inputs {
    match input {
      FnArg::Receiver(receiver) => {
        return Error::new_spanned(receiver, "`#[route]` handlers cannot take `self`")
          .to_compile_error()
          .into();
      }
      FnArg::Typed(input) => types.push((*input.ty).clone()),
    }
  }

  let output = match &item.sig.output {
    ReturnType::Default => syn::parse_quote!(()),
    ReturnType::Type(_, ty) => (**ty).clone(),
  };

  quote! {
    #item

    pub(crate) fn #route(registry: &mut deco::Registry) -> Route {
      let mut endpoint = crate::reference::Endpoint::default();
      #(<#types as Describe>::describe(&mut endpoint, registry);)*
      <#output as Describe>::describe(&mut endpoint, registry);
      Route {
        endpoint,
        method: axum::http::Method::#method,
        path: #path,
        router: axum::routing::on(axum::routing::MethodFilter::#method, #name),
      }
    }
  }
  .into()
}
