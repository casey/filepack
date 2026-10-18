Lints
=====

| lint | description |
|------|-------------|
%% for (lint, description) in self.lints {
{{ Row((format!("`{lint}`"), description)) }}
%% }

Groups
------

| group | description | lints |
|-------|-------------|-------|
%% for (name, group) in self.groups {
{{ Row((format!("`{name}`"), &group.description, list(&group.lints))) }}
%% }
