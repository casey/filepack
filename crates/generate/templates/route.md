# {{ route_path(self.path) }}

%% for (method, endpoint) in &self.route.methods {
## {{ method }}

{{ endpoint.description }}

| property | value |
|----------|-------|
| authorization required | {{ endpoint.authenticated }} |
| range requests supported | {{ endpoint.range }} |
%%   if !endpoint.parameters.is_empty() {

### Parameters

| parameter | type | description |
|-----------|------|-------------|
%%     for (index, (parameter, kind)) in self.route.parameters.iter().zip(&endpoint.parameters).enumerate() {
%%       if self.route.wildcard && index + 1 == self.route.parameters.len() {
{{ Row((format!("`*{}`", parameter.name), path_parameter_type(*kind), &parameter.description)) }}
%%       } else {
{{ Row((format!("`{}`", parameter.name), path_parameter_type(*kind), &parameter.description)) }}
%%       }
%%     }
%%     if self.route.wildcard {

A parameter beginning with `*` is a wildcard. Wildcard parameters match the
remainder of the path, including any slashes, and must not be empty.
%%     }
%%   }
%%   if !endpoint.query.is_empty() {

### Query Parameters

| parameter | default | description |
|-----------|---------|-------------|
%%     for parameter in &endpoint.query {
{{ Row((format!("`{}`", parameter.name), format!("`{}`", parameter.default), &parameter.description)) }}
%%     }
%%     for parameter in &endpoint.query {

#### `{{ parameter.name }}`

| value | description |
|-------|-------------|
%%       for value in &parameter.values {
{{ Row((format!("`{}`", value.value), &value.description)) }}
%%       }
%%     }
%%   }
%%   if let Some(request) = &endpoint.request {

### Request Body

%%     match request {
%%       Request::Deco { fields, limit } => {
A deco-encoded struct of at most {{ Count::new(*limit, "byte") }}:

{{ DecoFieldsMd { fields, registry: self.registry } }}
%%       }
%%       Request::Bytes => {
An arbitrary byte string.
%%       }
%%     }
%%   }
%%   if let Some(response) = &endpoint.response {

### Response Body

%%     match response {
%%       Response::Deco { fields } => {
A deco-encoded struct:

{{ DecoFieldsMd { fields, registry: self.registry } }}
%%       }
%%       Response::File => {
The contents of a file.
%%       }
%%       Response::Html => {
An HTML page.
%%       }
%%       Response::Redirect => {
A redirect.
%%       }
%%       Response::Static => {
A static asset.
%%       }
%%     }
%%   }

%% }
