# Jumper

Jumper is a command line tool for quick navigation around your filesystem.
Unlike many other tools it doesn't try to guess where you would like to be. You
have to manually select each folder you would like to appear on the quick jump
list.

It leverages the great `fzf` tool for fuzzy searching of target directory.

## Usage

First you need to select the folders you want to later jump to by using the
`save` command:

```bash
jumper save          # this saves current directory under it's name
jumper save other    # this saves current directory under 'other' name
```

Later you can see all your bookmarked folders using `jumper list` and retrieve
specific folder with `jumper get <name>`.

## Shell setup

Jumper by itself is just a bookmarking tool. To be able to actually jump to
selected path you need to configure the shell. Below are the definitions of 
`jg` command that will perform the jump. When invoked without any argument
it uses `fzf` to search for a path.

*Windows Powershell*

Add this to your `$profile` file:

```powershell
function Jump-Location {
    if ([string]::IsNullOrEmpty($args)) {
        $selected = $(jumper list | fzf --ansi).Trim().Split(' ')[-1]
        Set-Location -Path $selected
    } else {
        Set-Location -Path $(jumper get $args)
    }
}
New-Alias jg Jump-Location
```

*Bash*

Add this to your `~/.bashrc`:

```bash
jumpLocation() {
    if [ -z "$@" ]; then
        selected=$(jumper list | fzf --ansi | awk -F'|' '{gsub(/^\s+/, "", $2); print $2}')
        cd $selected
    else
        selected=$(jumper get $@)
        cd $selected
    fi
}
alias jg="jumpLocation"
```
