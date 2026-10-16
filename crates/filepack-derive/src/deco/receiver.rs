use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Receiver {
  Binding,
  Field,
}

impl Receiver {
  pub(crate) fn base(self, ident: &Ident) -> proc_macro2::TokenStream {
    match self {
      Self::Binding => quote! { #ident },
      Self::Field => quote! { self.#ident },
    }
  }

  pub(crate) fn reference(self, ident: &Ident) -> proc_macro2::TokenStream {
    match self {
      Self::Binding => quote! { #ident },
      Self::Field => quote! { &self.#ident },
    }
  }
}
