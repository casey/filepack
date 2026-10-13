Lints
=====

| lint |
|------|
%% for lint in self.lints {
| `{{ lint }}` |
%% }

Groups
------

| group | lints |
|-------|-------|
%% for (group, lints) in self.groups {
| `{{ group }}` | {{ list(lints) }} |
%% }
