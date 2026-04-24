use {
  darling::{FromDeriveInput, FromField, ast},
  proc_macro::TokenStream,
  quote::quote,
  syn::{DeriveInput, LitInt},
};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
struct Input {
  data: ast::Data<(), Field>,
  ident: syn::Ident,
}

#[derive(FromField)]
#[darling(forward_attrs(n))]
struct Field {
  attrs: Vec<syn::Attribute>,
  ident: Option<syn::Ident>,
  ty: syn::Type,
}

struct ParsedField<'a> {
  ident: &'a syn::Ident,
  n: u8,
  optional: bool,
}

impl Field {
  fn n(&self) -> syn::Result<u8> {
    let ident = self.ident.as_ref().unwrap();

    let mut n = None;

    for attr in &self.attrs {
      if attr.path().is_ident("n") {
        if n.is_some() {
          return Err(syn::Error::new_spanned(attr, "duplicate #[n] attribute"));
        }
        let lit = attr.parse_args::<LitInt>()?;
        n = Some(lit.base10_parse::<u8>()?);
      }
    }

    n.ok_or_else(|| syn::Error::new_spanned(ident, "missing #[n(N)] attribute"))
  }

  fn is_option(&self) -> bool {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = &self.ty {
      path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == "Option"
    } else {
      false
    }
  }

  fn parse(&self) -> syn::Result<ParsedField> {
    let ident = self.ident.as_ref().unwrap();
    let n = self.n()?;
    let optional = self.is_option();
    Ok(ParsedField { ident, n, optional })
  }
}

#[proc_macro_derive(Encode, attributes(n))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.derive_encode_inner() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

#[proc_macro_derive(Decode, attributes(n))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  let input = match Input::from_derive_input(&input) {
    Ok(input) => input,
    Err(err) => return err.write_errors().into(),
  };

  match input.derive_decode_inner() {
    Ok(tokens) => tokens.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

impl Input {
  fn derive_decode_inner(&self) -> syn::Result<proc_macro2::TokenStream> {
    let name = &self.ident;
    let fields = self.parse_fields()?;

    let decode_fields = fields.iter().map(|f| {
      let ident = f.ident;
      let n = f.n;
      if f.optional {
        quote! { let #ident = map.optional_key(#n)?; }
      } else {
        quote! { let #ident = map.required_key(#n)?; }
      }
    });

    let field_names = fields.iter().map(|f| f.ident);

    Ok(quote! {
      impl crate::Decode for #name {
        fn decode(decoder: &mut crate::Decoder) -> Result<Self, crate::DecodeError> {
          let mut map = decoder.map::<u8>()?;
          #(#decode_fields)*
          map.finish()?;
          Ok(Self {
            #(#field_names,)*
          })
        }
      }
    })
  }

  fn parse_fields(&self) -> syn::Result<Vec<ParsedField<'_>>> {
    let fields = self.data.as_ref().take_struct().unwrap();

    let mut parsed = fields
      .into_iter()
      .map(|field| field.parse())
      .collect::<syn::Result<Vec<ParsedField>>>()?;

    parsed.sort_by_key(|f| f.n);

    for window in parsed.windows(2) {
      if window[0].n == window[1].n {
        return Err(syn::Error::new_spanned(
          window[1].ident,
          format!("duplicate key {}", window[1].n),
        ));
      }
    }

    for (i, field) in parsed.iter().enumerate() {
      if field.n != i as u8 {
        return Err(syn::Error::new_spanned(
          field.ident,
          format!(
            "keys must be contiguous starting from 0: expected {i}, found {}",
            field.n
          ),
        ));
      }
    }

    Ok(parsed)
  }

  fn derive_encode_inner(&self) -> syn::Result<proc_macro2::TokenStream> {
    let name = &self.ident;
    let fields = self.parse_fields()?;

    let required_count = fields.iter().filter(|f| !f.optional).count() as u64;

    let count_optionals = fields.iter().filter(|f| f.optional).map(|f| {
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
      impl crate::Encode for #name {
        fn encode(&self, encoder: &mut crate::Encoder) {
          let length = #required_count #(#count_optionals)*;
          let mut map = encoder.map::<u8>(length);
          #(#items)*
        }
      }
    })
  }
}
