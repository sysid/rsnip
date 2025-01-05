# rsnip.bash

complete -r rsnip     # Remove any existing 'rsnip' completion
complete -r mytags    # Remove any existing 'mytags' completion

# 1) Completion function invoked by bash when completing `rsnip`
_rsnip_complete() {
  # Current word being typed
  local cur="${COMP_WORDS[COMP_CWORD]}"

  # Use a local IFS so space/newline handling won't break COMPREPLY
  local IFS=$'\n'
  # Call your Rust binary in "scriptable" mode that prints all possible completions
  local output
  # Uses the hidden subcommand "complete"
  output=$(rsnip complete --ctype mytype --input "${cur}" --scriptable-output 2>/dev/null)

  # Split the returned output into an array of suggestions
  COMPREPLY=( $(compgen -W "${output}" -- "${cur}") )
  return 0
}

complete -F _rsnip_complete rsnip --ctype mytpye

# For a different type:
_rsnip_complete_tags() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local IFS=$'\n'
  local output
  output=$(rsnip complete --ctype tags --input "${cur}" --scriptable-output 2>/dev/null)

  COMPREPLY=( $(compgen -W "${output}" -- "${cur}") )
  return 0
}

complete -F _rsnip_complete_tags mytags
