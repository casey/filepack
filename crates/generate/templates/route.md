# {{ escape(self.path) }}

%% for (method, endpoint) in &self.route.methods {
## {{ method }}

%%   if !endpoint.parameters.is_empty() {
| parameter | type |
|-----------|------|
%%     for (name, kind) in self.route.parameters.iter().zip(&endpoint.parameters) {
| `{{ name }}` | {{ parameter(*kind) }} |
%%     }

%%   }
%%   if endpoint.authenticated {
Requires authorization.
%%   } else {
Does not require authorization.
%%   }

%% }
