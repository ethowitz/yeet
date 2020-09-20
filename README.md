# yeet
A tool to yeet files into the dumpster. I wrote this simple tool to:
1. Get better at Rust
2. Give myself some basic "move files to the trash" functionality on the command line
3. Use the word "yeet" in a public GitHub repository

## Usage
As with any filesystem tool, `yeet` may have bugs and has the potential to remove every
file from your disk. So use with caution!

```
NAME
        yeet - yeet files into the dumpster
SYNOPSIS
        yeet [OPTION]... [FILE]...

DESCRIPTION
        `yeet` is a command line tool that allows you to move files to the "dumpster", which is the
        `$HOME/.dumpster` directory. What's unique about this tool is that, rather than dumping
        files in the folder flatly, it preserves the directory structure of the original file
        relative to `$HOME`. For example, if we have a file located at `$HOME/foo/bar/i_am_a_file`
        and we run `yeet i_am_a_file` from inside `$HOME/foo/bar/`, `yeet` will create `foo` and
        `bar` directories before moving the file. In this case, `i_am_a_file` would be moved to
        `$HOME/.dumpster/foo/bar/i_am_a_file`.

        Since the original location of `i_am_a_file` is "preserved" in the directory hierarchy of
        the dumpster, we can restore the file to its original location (provided the enclosing
        directory (`$HOME/foo/bar/`) still exists) by running
        `yeet --restore $HOME/.dumpster/foo/bar/i_am_a_file` (we can specify relative directories
        here as well).

        If a file at a given directory already exists in the dumpster, `yeet` will append a suffix
        to the filename, which avoids unintentional loss of data.

OPTIONS
        --empty        permanently removes all files and directories from the dumpster

        --restore      restores the specified files back to their original locations, assuming the
                       enclosing directories still exist; expects paths to files currently in the
                       dumpster
```

