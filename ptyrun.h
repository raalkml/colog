#ifndef _PTYRUN_H_
#define _PTYRUN_H_ 1

struct hook
{
    /* 0 - parsed and printed, !0 - pass to the next handler */
    int (*proc)(int fd_id /* 1 - stdin, 2 - stderr */, char *line, int len);
};

int ptyrun(char *argv[], struct hook *hooks, int hook_count, int merge);
void show_params(char * const *argv, int fd);

extern int verbose;
extern int restart;

#endif /* _PTYRUN_H_ */
