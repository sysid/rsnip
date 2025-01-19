rsnip
# generate configuration
rsnip  --generate-config

# edit configuration
vim /Users/tw/.config/rsnip/config.toml
# define your shortcuts (e.g. commma)

# edit shell snippets
rsnip edit --ctype shell

# alternative to edit shell snippets: use alias
# e, hel<tab>  # jump to snippet to edit
e, hel

# list existing snippets
rsnip list --ctype shell

# copy snippet to clipbord for further use
# type: , hel<tab>
, hel
# snippet is now in clipboard: CTRL-V

# have fun
