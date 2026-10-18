Lints
=====

| lint | description |
|------|-------------|
%% for (lint, description) in self.lints {
| `{{ lint }}` | {{ description }} |
%% }

Groups
------

| group | description | lints |
|-------|-------------|-------|
%% for (name, group) in self.groups {
| `{{ name }}` | {{ group.description }} | {{ list(&group.lints) }} |
%% }
