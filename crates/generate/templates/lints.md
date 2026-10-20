Lints
=====

Groups
------

| group | description | lints |
|-------|-------------|-------|
%% for (name, group) in self.groups {
{{ Row((format!("`{name}`"), &group.description, list(&group.lints))) }}
%% }

Lints
-----

| lint | description |
|------|-------------|
%% for (lint, description) in self.lints {
{{ Row((format!("`{lint}`"), description)) }}
%% }

Junk Files
----------

The `junk` lint fails if the package contains a file with one of the following
names: {{ list(self.junk_names) }}.

Windows Reserved Characters
---------------------------

The `windows-reserved-character` lint fails if a filename contains one of the
following characters: {{ list(self.windows_reserved_characters) }}.

Windows Reserved Names
----------------------

The `windows-reserved-filename` lint fails if a filename , ignoring case, is
one of the following names, or one of the following names followed by a period:
{{ list(self.windows_reserved_names) }}.
