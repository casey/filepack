Web
===

The Filepack server responds to the following web routes:

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

And the following fallback routes, if no other route matches:

| method | path | description |
|--------|------|-------------|
| GET | `/{fingerprint}` | Redirect to [`/package/{fingerprint}`](web/package-package.md) |
| GET | `/{number}` | Redirect to [`/package/{number}`](web/package-package.md) |
| GET | `/{revision}` | Redirect to [`/package/{revision}`](web/package-package.md) |
| GET | `/{name}/` | Serve `static/index.html` of mounted web package with `name` |
| GET | `/{name}/{*path}` | Serve file at `static/{path}` of mounted web package with `name` |
