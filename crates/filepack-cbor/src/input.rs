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

  pub(crate) fn derive_encode_inner(&self) -> Result<proc_macro2::TokenStream> {
    let name = &self.ident;
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

  fn parse_fields(&self) -> Result<Vec<ParsedField>> {
    let fields = self
      .data
      .as_ref()
      .take_struct()
      .unwrap()
      .into_iter()
      .map(Field::parse)
      .collect::<Result<Vec<ParsedField>>>()?;

    let mut n = HashSet::new();
    for (i, field) in fields.iter().enumerate() {
      if !n.insert(field.n) {
        return Err(syn::Error::new_spanned(
          field.ident,
          format!("duplicate key {}", field.n),
        ));
      }

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

    Ok(fields)
  }
}
