# `{{ self.name }}`

```
{{ self.help }}
```
%% if let Some(output) = self.output {

Prints {{ link(Chapter::Json, output) }} as JSON.
%% }
