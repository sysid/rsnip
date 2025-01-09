# rsnip.bash
complete -r rsnip > /dev/null 2>&1  # Remove any existing completion
complete -r "," > /dev/null 2>&1     # Remove any existing completion for the alias

# Enhanced completion function
_rsnip_complete() {
    local cur prev cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Detect if we're using the alias or full command
    if [[ ${COMP_WORDS[0]} == "," ]]; then
        :
    else
        # List of available commands
        local commands="copy edit list types"

        # If completing a command
        if [[ ${COMP_CWORD} -eq 1 ]]; then
            COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
            return 0
        fi
    fi

    # Get available snippet types
    local snippet_types=$(rsnip --info 2>/dev/null | grep "Available types:" | cut -d':' -f2)

    case "${prev}" in
        "--ctype"|"-c")
            COMPREPLY=( $(compgen -W "${snippet_types}" -- ${cur}) )
            return 0
            ;;
        "complete"|"copy"|"xxx")
            COMPREPLY=( $(compgen -W "--ctype --input --interactive" -- ${cur}) )
            return 0
            ;;
    esac

    # Handle the alias case - treat everything after "," as input
    if [[ ${COMP_WORDS[0]} == "," ]]; then
        # Save current terminal state
        tput smcup

        local result
        result="$(rsnip complete --interactive --ctype shell --input "${cur}")"

        # Restore terminal state
        tput rmcup

        if [[ -n "$result" ]]; then
            COMPREPLY=("$result")
            # Redraw line after fzf closes
            printf '\e[5n'
        fi
        return 0
    fi

    # If we're completing an input and have a type specified
    local ctype=""
    for ((i=1; i<COMP_CWORD; i++)); do
        if [[ "${COMP_WORDS[i]}" == "--ctype" || "${COMP_WORDS[i]}" == "-c" ]]; then
            ctype="${COMP_WORDS[i+1]}"
            break
        fi
    done

    if [[ -n "${ctype}" && "${prev}" == "--input" ]]; then
        # Save current terminal state
        tput smcup

        local result
        result="$(rsnip complete --interactive --ctype "${ctype}" --input "${cur}")"

        # Restore terminal state
        tput rmcup

        if [[ -n "$result" ]]; then
            COMPREPLY=("$result")
            # Redraw line after fzf closes
            printf '\e[5n'
        fi
    fi
}

# Enable line editing features needed for interactive completion
if [[ :"${SHELLOPTS}": =~ :(vi|emacs): && ${TERM} != 'dumb' ]]; then
    # Bind escape sequence for redrawing line after fuzzy selection
    bind '"\e[0n": redraw-current-line' &>/dev/null
fi

# Setup completion for both the command and the alias
complete -F _rsnip_complete rsnip
complete -F _rsnip_complete ","