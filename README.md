# colog, a simple log colorizer

An extremely simple (but small) log colorizer which uses regular expressions
to split a fixed, compiled-in, set of log file formats, insert a very limited
set of ANSI terminal escape codes between the parts, and output it all to the
terminal.

The special thing about this program is that it runs the command in a PTY, so
that stdio routines do not activate fully buffered output on `stdout`.

The short list of supported formats and their colorizations is in `regex_colors.c` file.

# Usage

```
colog [-v -c <filter> --] executable args...

-v  Verbose diagnostics.

-c <filter>
    Use this command (passed to `sh -c`) to process all uncolorized output.
```

For example:

    colog tail -f /var/log/syslog

