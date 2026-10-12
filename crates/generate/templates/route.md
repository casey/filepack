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

%% }
