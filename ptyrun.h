/*
 * A log colorizer for systems with PTY support
 *
 * Compatibility routines for PTY handling.
 */
#ifndef _PTYRUN_H_
#define _PTYRUN_H_ 1
#include <unistd.h>

struct hook
{
    /* 0 - parsed and printed, !0 - pass to the next handler */
    int (*proc)(int fd_id /* 1 - stdin, 2 - stderr */, char *line, int len);
};

int ptyrun(char *argv[], struct hook *hooks, int hook_count, int merge, pid_t *child);
void show_params(char * const *argv, int fd);

#endif /* _PTYRUN_H_ */
