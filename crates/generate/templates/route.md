# {{ escape(self.path) }}

%% for (method, endpoint) in &self.route.methods {
## {{ method }}

%%   if !endpoint.parameters.is_empty() {
| parameter | type |
|-----------|------|
%%     for (index, (name, kind)) in self.route.parameters.iter().zip(&endpoint.parameters).enumerate() {
%%       if self.route.wildcard && index + 1 == self.route.parameters.len() {
| `*{{ name }}` | {{ parameter(*kind) }} |
%%       } else {
| `{{ name }}` | {{ parameter(*kind) }} |
%%       }
%%     }

%%     if self.route.wildcard {
A parameter beginning with `*` is a wildcard. Wildcard parameters match the
remainder of the path, including any slashes, and must not be empty.

%%     }
%%   }
%%   if endpoint.authenticated {
Requires authorization.
%%   } else {
Does not require authorization.
%%   }
%%   if endpoint.range {

Supports range requests.
%%   }
%%   if let Some(request) = &endpoint.request {

### Request Body

%%     match request {
%%       Request::Deco { limit, schema } => {
A deco-encoded value of at most {{ Count::new(*limit, "byte") }}.

{{ DefinitionMd { schema } }}
%%       }
%%       Request::Bytes => {
An arbitrary byte string.
%%       }
%%     }
%%   }
%%   if let Some(response) = &endpoint.response {
%%     match response {
%%       Response::Deco { schema } => {

### Response Body

A deco-encoded value.

{{ DefinitionMd { schema } }}
%%       }
%%       Response::File => {

### Response Body

The contents of a file.
%%       }
%%       Response::Html => {

### Response Body

An HTML page.
%%       }
%%       Response::Redirect => {

### Response Body

A redirect.
%%       }
%%       Response::Static => {

### Response Body

A static asset.
%%       }
%%     }
%%   }

%% }
