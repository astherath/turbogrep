# `turbogrep`
really tiny (and silly) cli to mimic find + exec + cat + grep calls

## Usage
```
turbogrep --help

USAGE:
    turbogrep [FLAGS] <expr> <old> <new>

FLAGS:
    -d, --dry-run    if set, does not execute the final step of replacing the matching terms in the files
    -h, --help       Prints help information
    -s, --silent     if set, does not print out any output except the final files seen/changed count
    -V, --version    Prints version information

ARGS:
    <expr>    the pattern expression to match the files for
    <old>     the (old) term currently present in the files to replace
    <new>     the (new) term to replace the old term with
```

## Purpose
`turbogrep` was made as a lightweight replacement for CLI-based refactoring tools.

Without the help of an IDE, it's hard and annoying to have to write scripts to refactor large directories at once while previewing the changes to be made,
which hopefully is the niche that `turbogrep` fills.



## Module layout

![image-20220216165814959](/Users/felipearce/Library/Application Support/typora-user-images/image-20220216165814959.png)



### Control flow summary 

- Main invokes the CLI to get input args (file, directory, flags, search term, etc.)
- CLI mod invokes the file parser in order to read the correct files 
  - the file parser invoker is responsible for handling things such as nested directories, non-readable files, etc.
- The file parser mod then groups all of the data into a struct that can be processed by the console printer