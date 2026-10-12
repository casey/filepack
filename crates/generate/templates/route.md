# {{ escape(self.path) }}

%% for (method, endpoint) in self.methods {
## {{ method }}

%%   if endpoint.authenticated {
Requires authorization.
%%   } else {
Does not require authorization.
%%   }

%% }
