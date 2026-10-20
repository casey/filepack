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

    for line in description.rest().unwrap_or_default().lines() {
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

    let description = description.summary();

    let name = &item.sig.ident;

    let route = format_ident!("{name}_route");

    let names = parameters.iter().map(|parameter| &parameter.0);

    let descriptions = parameters.iter().map(|parameter| &parameter.1);

    quote! {
      #item

      pub(crate) fn #route(registry: &mut deco::Registry) -> Route {
        let mut endpoint = crate::reference::Endpoint::default();
        endpoint.description = #description.into();
        #(<#types as Describe>::describe(&mut endpoint, registry);)*
        <#output as Describe>::describe(&mut endpoint, registry);
        Route {
          endpoint,
          method: axum::http::Method::#method,
          parameters: vec![#(
            crate::reference::PathParameter {
              description: #descriptions.into(),
              name: #names.into(),
            }
          ),*],
          path: #path,
          router: axum::routing::on(axum::routing::MethodFilter::#method, #name),
        }
      }
    }
  }
}
