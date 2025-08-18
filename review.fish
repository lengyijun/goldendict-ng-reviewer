complete -c review -l help           -d 'Print the help message.' -f
complete -c review -l frequency      -d 'Frequent words will be reviewed.' -f
complete -c review -l merriam        -d 'merriam' -f
complete -c review -l no-extend      -d 'Never extend' -f
complete -c review -l random         -d 'Choose next word to review randomly' -f
complete -c review -l last-no-row    -d 'Only review recently updated words' -f

set -l subcommands favourite phrase (cat ~/.config/goldendict/favorites | grep folder | grep expanded | grep -oP '(?<=name=").+?(?=")')

complete -f -c review -a (string join " " $subcommands)

