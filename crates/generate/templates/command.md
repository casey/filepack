# `{{ self.name }}`

```
{{ self.help }}
```
%% if let Some(input) = self.input {
%%   if self.name == "filepack archive" {

Reads {{ description(Chapter::Json, self.registry, input) }} as JSON from `<INPUT>`.
%%   } else {
%%     unreachable!()
%%   }
%% }
%% if let Some(output) = self.output {
%%   if self.name == "filepack verify" {

Prints {{ description(Chapter::Json, self.registry, output) }} as JSON to standard output if `--print` is passed.
%%   } else {

Prints {{ description(Chapter::Json, self.registry, output) }} as JSON to standard output.
%%   }
%% }
