API
===

The Filepack server responds to the following API routes:

| method | path | description |
|--------|------|-------------|
%% for (path, page) in self.pages {
%%   for (method, endpoint) in &self.routes[*path].methods {
{{ Row((
  method,
  route_link(path, page),
  &endpoint.description,
)) }}
%%   }
%% }
