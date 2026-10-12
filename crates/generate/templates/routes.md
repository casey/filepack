Routes
======

The filepack server responds to the following API routes:

| method | path |
|--------|------|
%% for (path, methods) in self.routes.iter().filter(|(path, _)| path.starts_with("/api/")) {
%%   for method in methods {
| {{ method }} | `{{ path }}` |
%%   }
%% }

And the following non-API routes:

| method | path |
|--------|------|
%% for (path, methods) in self.routes.iter().filter(|(path, _)| !path.starts_with("/api/")) {
%%   for method in methods {
| {{ method }} | `{{ path }}` |
%%   }
%% }
