use super::*;

pub(crate) struct ParsedField<'a> {
  pub(crate) decode_with: Option<Path>,
  pub(crate) encode_with: Option<Path>,
  pub(crate) ident: &'a Ident,
  pub(crate) n: u64,
  pub(crate) optional: bool,
}

impl ParsedField<'_> {
  pub(crate) fn decode(fields: &[Self]) -> Vec<proc_macro2::TokenStream> {
    fields
      .iter()
      .map(|field| {
        let ident = field.ident;
        let n = field.n;
        match (&field.decode_with, field.optional) {
          (Some(path), true) => quote! { let #ident = map.optional_key_with(#n, #path)?; },
          (Some(path), false) => quote! { let #ident = map.required_key_with(#n, #path)?; },
          (None, true) => quote! { let #ident = map.optional_key(#n)?; },
          (None, false) => quote! { let #ident = map.required_key(#n)?; },
        }
      })
      .collect()
  }

  pub(crate) fn encode(
    fields: &[Self],
    receiver: Receiver,
  ) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    let required = fields
      .iter()
      .filter(|field| !field.optional)
      .count()
      .into_u64();

    let optional = fields.iter().filter(|field| field.optional).map(|field| {
      let base = receiver.base(field.ident);
      quote! { + u64::from(#base.is_some()) }
    });

    let length = quote! { #required #(#optional)* };

    let items = fields
      .iter()
      .map(|field| {
        let n = field.n;
        let base = receiver.base(field.ident);
        let reference = receiver.reference(field.ident);
        match (&field.encode_with, field.optional) {
          (Some(path), true) => quote! { map.optional_item_with(#n, #base.as_ref(), #path); },
          (Some(path), false) => quote! { map.item_with(#n, #reference, #path); },
          (None, true) => quote! { map.optional_item(#n, #base.as_ref()); },
          (None, false) => quote! { map.item(#n, #reference); },
        }
      })
      .collect();

    (length, items)
  }
}
