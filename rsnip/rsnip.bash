# rsnip.bash
complete -r rsnip  # Remove any existing completion

# Enhanced completion function with both modes
_rsnip_complete() {
  # Only show completions when cursor is at the end of the line
  [[ ${#COMP_WORDS[@]} -eq $((COMP_CWORD + 1)) ]] || return

  local result
  # If there is a space after the last word, use interactive with previous word as input
  if [[ -z ${COMP_WORDS[-1]} ]] && [[ ${#COMP_WORDS[@]} -gt 1 ]]; then
    result="$(rsnip complete --interactive --ctype mytype --input "${COMP_WORDS[-2]}")" &&
      COMPREPLY=("$result")
    # Redraw line after fzf closes
    printf '\e[5n'
  else
    # Use non-interactive completion for direct input
    result="$(rsnip complete --ctype mytype --input "${COMP_WORDS[-1]}")"
    if [[ -n "$result" ]]; then
      COMPREPLY=("$result")
    fi
  fi
}

# Enable line editing features needed for interactive completion
if [[ :"${SHELLOPTS}": =~ :(vi|emacs): && ${TERM} != 'dumb' ]]; then
  # Bind escape sequence for redrawing line after fuzzy selection
  bind '"\e[0n": redraw-current-line' &>/dev/null
fi

# Setup completion
complete -F _rsnip_complete rsnip