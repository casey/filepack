use {
  self::{
    attributes::Attributes, container_attribute::ContainerAttribute, field::Field, input::Input,
    parsed_field::ParsedField, parsed_variant::ParsedVariant, receiver::Receiver,
    route_attribute::RouteAttribute, variant::Variant,
  },
  darling::{FromDeriveInput, FromField, FromVariant, ast::Data, ast::Fields},
  heck::{ToKebabCase, ToSnakeCase},
  proc_macro::TokenStream,
  quote::{format_ident, quote},
  std::collections::HashSet,
  strum::{Display, EnumString, IntoStaticStr},
  syn::{
    Attribute, DeriveInput, Error, FnArg, GenericArgument, Generics, Ident, ItemFn, LitInt, LitStr,
    Path, PathArguments, ReturnType, Token, Type, TypeParamBound, TypePath,
    meta::ParseNestedMeta,
    parse::{Parse, ParseBuffer, ParseStream},
  },
  usized::IntoU64,
};

mod attributes;
mod container_attribute;
mod field;
mod input;
mod parsed_field;
mod parsed_variant;
mod receiver;
mod route_attribute;
mod variant;
mod yaml;

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[proc_macro_derive(Decode, attributes(deco, n))]
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

fn display_name(ident: &Ident) -> String {
  ident.to_string().to_snake_case().replace('_', " ")
}

#[proc_macro_derive(Encode, attributes(deco, n))]
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

#[proc_macro_derive(Magic, attributes(deco, n))]
pub fn magic(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.magic() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

fn number(ident: &Ident, attributes: &[Attribute]) -> Result<u64> {
  let mut n = None;

  for attribute in attributes {
    if attribute.path().is_ident("n") {
      if n.is_some() {
        return Err(Error::new_spanned(attribute, "duplicate `#[n]` attribute"));
      }
      n = Some(attribute.parse_args::<LitInt>()?.base10_parse::<u64>()?);
    }
  }

  n.ok_or_else(|| Error::new_spanned(ident, "missing `#[n(N)]` attribute"))
}

fn option_inner(ty: &Type) -> Option<&Type> {
  if let Type::Path(TypePath {
    attrs: _,
    path,
    qself: None,
  }) = ty
    && path.leading_colon.is_none()
    && path.segments.len() == 1
    && path.segments[0].ident == "Option"
    && let PathArguments::AngleBracketed(arguments) = &path.segments[0].arguments
    && arguments.args.len() == 1
    && let GenericArgument::Type(ty) = &arguments.args[0]
  {
    Some(ty)
  } else {
    None
  }
}

fn path_value(meta: &ParseNestedMeta, placeholder: &str) -> Result<Path> {
  let name = meta.path.require_ident()?;
  meta.value().and_then(ParseBuffer::parse).map_err(|_| {
    meta.error(format!(
      "`#[deco({name})]` must be of the form `#[deco({name} = {placeholder})]`"
    ))
  })
}

#[proc_macro_attribute]
pub fn route(attribute: TokenStream, item: TokenStream) -> TokenStream {
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

fn validate_numbers<'a>(start: u64, numbers: impl IntoIterator<Item = (&'a Ident, u64)>) -> Result {
  let mut seen = HashSet::new();

  for (i, (ident, n)) in numbers.into_iter().enumerate() {
    if !seen.insert(n) {
      return Err(Error::new_spanned(
        ident,
        format!("duplicate `#[n({n})]` attribute"),
      ));
    }

    let expected = start + i.into_u64();

    if n != expected {
      return Err(Error::new_spanned(
        ident,
        format!(
          "`#[n]` attributes must be contiguous starting from {start}: expected {expected}, found {n}"
        ),
      ));
    }
  }

  Ok(())
}

#[proc_macro_derive(Yaml)]
pub fn yaml(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match yaml::Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.schema() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}
