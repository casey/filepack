#compdef filepack
source <(FILEPACK_COMPLETE=zsh filepack)
if [ "$funcstack[1]" = "_filepack" ]; then
  _clap_dynamic_completer_filepack "$@"
fi
