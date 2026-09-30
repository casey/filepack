use super::*;

pub(crate) struct ParsedField<'a> {
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
        if field.optional {
          quote! { let #ident = map.optional_key(#n)?; }
        } else {
          quote! { let #ident = map.required_key(#n)?; }
        }
      })
      .collect()
  }

  pub(crate) fn encode(fields: &[Self], receiver: Receiver) -> Vec<proc_macro2::TokenStream> {
    fields
      .iter()
      .rev()
      .map(|field| {
        let n = field.n;
        if field.optional {
          let base = receiver.base(field.ident);
          quote! { map.optional_item(#n, #base.as_ref()); }
        } else {
          let reference = receiver.reference(field.ident);
          quote! { map.item(#n, #reference); }
        }
      })
      .collect()
  }
}
