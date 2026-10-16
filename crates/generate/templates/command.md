# `{{ self.name }}`

```
{{ self.help }}
```
%% if let Some(input) = self.input {

Reads {{ link(Chapter::Json, input) }} as JSON from standard input.
%% }
%% if let Some(output) = self.output {

Prints {{ link(Chapter::Json, output) }} as JSON to standard output.
%% }
