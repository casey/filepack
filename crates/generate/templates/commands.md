Commands
========

| command | description |
|---------|-------------|
%% for (name, (command, path)) in self.pages {
| [{{ name }}]({{ path }}) | {{ command.about }} |
%% }
