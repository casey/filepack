Routes
======

The filepack server responds to the following API routes:

| method | path | description |
|--------|------|-------------|
%% for (path, route) in self.routes.iter().filter(|(_, route)| route.api) {
%%   for (method, endpoint) in &route.methods {
{{ Row((
  method,
  route_link(path, &self.pages[path.as_str()]),
  &endpoint.description,
)) }}
%%   }
%% }

And the following web routes:

| method | path | description |
|--------|------|-------------|
%% for (path, route) in self.routes.iter().filter(|(_, route)| !route.api) {
%%   for (method, endpoint) in &route.methods {
{{ Row((
  method,
  route_link(path, &self.pages[path.as_str()]),
  &endpoint.description,
)) }}
%%   }
%% }
