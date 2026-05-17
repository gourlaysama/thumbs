% THUMBS(1) Version 0.5.0 | Thumbs Usage Documentation

NAME
====

**thumbs** — manage cached thumbnails for files

SYNOPSIS
========

| **thumbs** **delete** \[_OPTIONS_] _FILE_...
| **thumbs** **locate** \[_OPTIONS_] _FILE_
| **thumbs** **cleanup** \[_OPTIONS_] \[_GLOB_]...
| **thumbs** **info**  \[_OPTIONS_] \[_FILE_]

DESCRIPTION
===========

Manage the cached thumbnails for files.

COMMANDS
========

delete _FILE_...
-----------------------------

Delete the thumbnails for the given files.

_FILE_...

:   File whose thumbnails to delete, or `"-"` for reading a file path from standard input.

    Multiple paths can be given, but only a single one can use `"-"` for standard input. Whatever can be read over standard input will be treated as a single path; use `"xargs"` or similar to programatically giving multiple paths.

locate _FILE_
---------------------------------

Print the path of thumbnails for the given files.

_FILE_

:   File whose thumbnails are to be found, or `"-"` to read a file path from standard input.

cleanup \[GLOB]...
-------------------------------------

Find thumbnails for files that no longer exist and optionally delete them.

_GLOB_...

:   Include or exclude files and directories that match the given globs. Can be used multiple times. Globbing rules match `".gitignore"` globs. Precede a glob with a `"!"` to exclude it.

    If no including glob is given, all files are included first before excluding globs are considered.

info \[_FILE_]
--------------

Show information about a file's thumbnails or the thumbnail cache.

_FILE_

:   A file to provide information about, or `"-"` to read a file path from standard input.

    If no file is given, global information about the thumbnail cache itself is returned.


OPTIONS
=======

Delete options
-------------

**-a**, **\--all**

:   Include hidden files and directories.

**-f**, **\--force**

:   Do not prompt and actually delete thumbnails.

    Running without **`-f/--force`** will never actually delete anything. If thumbs can detect that the terminal is interactive, it will prompt for deletion. Otherwise it will just print a summary of the operation and ask to rerun with **`-f/--force`**.

**-l,** **\--last-accessed** _LAST\_ACCESSED_

:   Only delete thumbnails for files that haven't been accessed since the given time.

    Can be either a RFC3339-like timestamp (`"2020-01-01 11:10:00"`) or a free-form duration like `"1year 15days 1week 2min"` or `"1h 6s 2ms"`.

**-r**, **\--recursive**

:   Recurse through directories.

Cleanup options
---------------

**-f**, **\--force**

:   Do not prompt and actually delete thumbnails.

    Running without **`-f/--force`** will never actually delete anything. If thumbs can detect that the terminal is interactive, it will prompt for deletion. Otherwise it will just print a summary of the operation and ask to rerun with **`-f/--force`**.


Global flags
------------

**-q**, **\--quiet**

:   Pass for less log output

**-v**, **\--verbose**

:   Pass for more log output

Info
----

-h, \--help

:   Print help information

-V, \--version

:   Print version information

EXIT STATUS
===========

0

:   The operation was successful.

1

:   There was an error.

125

:   The operation was successful because no files were acted upon at all.

ENVIRONMENT
===========

_`$NO_COLOR`_

:   If set to anything but `"0"`, disable color output.

_`$THUMBS_SYSLOG_PREFIXED`_

:   If set to anything but `"0"`, use the syslog format for log output.

BUGS
====

See GitHub Issues: <https://github.com/gourlaysama/thumbs/issues>

AUTHOR
======

Antoine Gourlay <antoine@gourlay.fr>
