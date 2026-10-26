Routes
======

The Filepack server responds to the following API routes:

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

And the following fallback routes, if no other route matches:

| method | path | description |
|--------|------|-------------|
| GET | `/{fingerprint}` | Redirect to [`/package/{fingerprint}`](routes/package-package.md) |
| GET | `/{number}` | Redirect to [`/package/{number}`](routes/package-package.md) |
| GET | `/{revision}` | Redirect to [`/package/{revision}`](routes/package-package.md) |
| GET | `/{name}/` | Serve `static/index.html` of mounted web package with `name` |
| GET | `/{name}/{*path}` | Serve file at `static/{path}` of mounted web package with `name` |
