// portable pty-preparser for wide range of compilers (gcc,HPaCC,SunCC,IBMxlC)
#ifndef __ptyrun_h__
#define __ptyrun_h__ 1

struct hook_t
{
    // 0 - parsed and printed, !0 - pass to the next handler
    int (*proc)(int /* 1 - stdin, 2 - stderr */, char *, int);
};

int ptyrun(char* argv[], hook_t * hooks, int hcount, int merge);
void show_params(const char * const * argv, int fd);
extern int verbose;
extern int restart;

#endif // __ptyrun_h__
