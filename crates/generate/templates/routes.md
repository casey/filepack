Routes
======

The filepack server responds to the following API routes:

| method | path |
|--------|------|
%% for (path, route) in self.routes.iter().filter(|(_, route)| route.api) {
%%   for method in route.methods.keys() {
| {{ method }} | [`{{ path }}`]({{ self.pages[path] }}) |
%%   }
%% }

And the following web routes:

| method | path |
|--------|------|
%% for (path, route) in self.routes.iter().filter(|(_, route)| !route.api) {
%%   for method in route.methods.keys() {
| {{ method }} | [`{{ path }}`]({{ self.pages[path] }}) |
%%   }
%% }
