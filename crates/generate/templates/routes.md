Routes
======

The filepack server responds to the following API routes:

| method | path |
|--------|------|
%% for (path, methods) in self.routes.iter().filter(|(path, _)| path.starts_with("/api/")) {
%%   for method in methods.keys() {
| {{ method }} | [`{{ path }}`]({{ self.pages[path] }}) |
%%   }
%% }

And the following web routes:

| method | path |
|--------|------|
%% for (path, methods) in self.routes.iter().filter(|(path, _)| !path.starts_with("/api/")) {
%%   for method in methods.keys() {
| {{ method }} | [`{{ path }}`]({{ self.pages[path] }}) |
%%   }
%% }
