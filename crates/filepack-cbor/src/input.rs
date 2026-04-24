use super::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub(crate) struct Input {
  data: Data<(), Field>,
  ident: Ident,
}

impl Input {
  pub(crate) fn derive_decode_inner(&self) -> Result<proc_macro2::TokenStream> {
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
          let mut map = decoder.map::<u64>()?;
          #(#decode_fields)*
          map.finish()?;
          Ok(Self {
            #(#field_names,)*
          })
        }
      }
    })
  }

  pub(crate) fn derive_encode_inner(&self) -> Result<proc_macro2::TokenStream> {
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
          let mut map = encoder.map::<u64>(length);
          #(#items)*
        }
      }
    })
  }

  fn parse_fields(&self) -> Result<Vec<ParsedField>> {
    let fields = self.data.as_ref().take_struct().unwrap();

    let mut parsed = fields
      .into_iter()
      .map(Field::parse)
      .collect::<Result<Vec<ParsedField>>>()?;

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
      if field.n != i.into_u64() {
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
}
