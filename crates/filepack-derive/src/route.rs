use super::*;

pub(crate) struct Route {
  description: Description,
  item: ItemFn,
  method: Ident,
  output: Type,
  parameters: Vec<(String, String)>,
  path: LitStr,
  types: Vec<Type>,
}

impl Route {
  pub(crate) fn new(attribute: RouteAttribute, item: ItemFn) -> Result<Self> {
    let RouteAttribute { method, path } = attribute;

    let ident = &item.sig.ident;

    let description = Description::new(ident, &item.attrs, "route", false, false)?;

    let mut parameters = Vec::<(String, String)>::new();

    for line in description.body().unwrap_or_default().lines() {
      let Some((name, description)) = line
        .strip_prefix('`')
        .and_then(|rest| rest.split_once("`: "))
      else {
        return Err(Error::new_spanned(
          ident,
          format!(
            "route parameter description `{line}` must be of the form `` `NAME`: DESCRIPTION ``"
          ),
        ));
      };

      Description::check_summary(ident, description, false)?;

      parameters.push((name.into(), description.into()));
    }

    let documented = parameters
      .iter()
      .map(|(name, _)| name.as_str())
      .collect::<Vec<&str>>();

    let expected = path
      .value()
      .split('/')
      .filter_map(|segment| segment.strip_prefix('{'))
      .map(|name| {
        name
          .strip_prefix('*')
          .unwrap_or(name)
          .strip_suffix('}')
          .unwrap()
          .to_string()
      })
      .collect::<Vec<String>>();

    if documented != expected {
      return Err(Error::new_spanned(
        ident,
        format!(
          "documented route parameters {documented:?} do not match path parameters {expected:?}"
        ),
      ));
    }

    let mut types = Vec::new();

    for input in &item.sig.inputs {
      match input {
        FnArg::Receiver(receiver) => {
          return Err(Error::new_spanned(
            receiver,
            "`#[route]` handlers cannot take `self`",
          ));
        }
        FnArg::Typed(input) => types.push((*input.ty).clone()),
      }
    }

    let output = match &item.sig.output {
      ReturnType::Default => syn::parse_quote!(()),
      ReturnType::Type(_, ty) => (**ty).clone(),
    };

    Ok(Self {
      description,
      item,
      method,
      output,
      parameters,
      path,
      types,
    })
  }

  pub(crate) fn tokens(&self) -> TokenStream {
    let Self {
      description,
      item,
      method,
      output,
      parameters,
      path,
      types,
    } = self;

    let summary = description.summary();

    let name = &item.sig.ident;

    let route = format_ident!("{name}_route");

    let names = parameters.iter().map(|parameter| &parameter.0);

    let descriptions = parameters.iter().map(|parameter| &parameter.1);

    let value = path.value();

    let api_path = if value.starts_with("/api/") {
      let api_path = value.strip_prefix("/api").unwrap();
      quote!(Some(#api_path))
    } else {
      quote!(None)
    };

    quote! {
      #item

      pub(crate) fn #route(
        registry: &mut ::filepack::__private::deco::Registry,
      ) -> ::filepack::__private::Route {
        let mut endpoint = ::filepack::__private::reference::Endpoint::default();
        endpoint.description = #summary.into();
        #(<#types as ::filepack::__private::Describe>::describe(&mut endpoint, registry);)*
        <#output as ::filepack::__private::Describe>::describe(&mut endpoint, registry);
        ::filepack::__private::Route {
          api_path: #api_path,
          endpoint,
          method: ::axum::http::Method::#method,
          parameters: vec![#(
            ::filepack::__private::reference::PathParameter {
              description: #descriptions.into(),
              name: #names.into(),
            }
          ),*],
          path: #path,
          router: ::axum::routing::on(
            ::axum::routing::MethodFilter::#method,
            #name,
          ),
        }
      }
    }
  }
}
