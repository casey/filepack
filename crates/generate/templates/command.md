# `{{ self.name }}`

```
{{ self.help }}
```
%% if let Some(input) = self.input {
%%   if self.name == "filepack archive" {

Reads {{ self.registry.description(Chapter::Json, input) }} as JSON from `<INPUT>`.
%%   } else {
%%     unreachable!()
%%   }
%% }
%% if let Some(output) = self.output {
%%   if self.name == "filepack verify" {

Prints {{ self.registry.description(Chapter::Json, output) }} as JSON to standard output if `--print` is passed.
%%   } else {

Prints {{ self.registry.description(Chapter::Json, output) }} as JSON to standard output.
%%   }
%% }
