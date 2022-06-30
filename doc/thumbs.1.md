% THUMBS(1) Version 0.4.4 | Thumbs Usage Documentation

NAME
====

**thumbs** — manage cached thumbnails for files

SYNOPSIS
========

| **thumbs** **delete** \[_OPTIONS_] \[_FILE_]...
| **thumbs** **locate** \[_OPTIONS_] \[_FILE_]
| **thumbs** **cleanup** \[_OPTIONS_] \[**-g**|**\--glob** glob]...
| **thumbs** \[**-h**|**\--help**|**-V**|**\--version**]

DESCRIPTION
===========

Manage the cached thumbnails for files.

ARGS
====

FILE

:   File whose thumbnail to operate upon. The file has to exist.

    This argument can be specified multiple times with the **delete** command.

OPTIONS
=======

Delete options
-------------

-f, \--force

:   Do not prompt and actually delete thumbnails.

    Running without **`-f/--force`** will never actually delete anything. If thumbs can detect that the terminal is interactive, it will prompt for deletion. Otherwise it will just print a summary of the operation and ask to rerun with **`-f/--force`**.

-l, \--last-accessed _LAST\_ACCESSED_

:   Only delete thumbnails for files that haven't been accessed since the given time.

    Can be either a RFC3339-like timestamp ('_`2020-01-01 11:10:00`_') or a free-form duration like '_`1year 15days 1week 2min`_' or '_`1h 6s 2ms`_'.

Cleanup Options
-----

-g, \--glob _GLOB_

:   Include or exclude files and directories that match the given globs.

    Globbing rules match .gitignore globs, like '_`*/foo.txt`_' or '_`num??.txt`_'. Precede a glob with a '_`!`_' (exclamation point) character to exclude anything that matches it.

    This option can be used multiple times. 

Global flags
------------

-a, \--all

:   Include hidden files and directories

-r, \--recursive

:   Recurse through directories

-q, \--quiet

:   Pass for less log output

-v, \--verbose

:   Pass for more log output

Info
----

-h, \--help

:   Print help information

-V, \--version

:   Print version information

BUGS
====

See GitHub Issues: <https://github.com/gourlaysama/thumbs/issues>

AUTHOR
======

Antoine Gourlay <antoine@gourlay.fr>
