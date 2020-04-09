// portable pty-preparser for wide range of compilers (gcc,HPaCC,SunCC,IBMxlC)
#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <stdlib.h>
#include <signal.h>

#include "ptyrun.h"
#include "output.h"
#include "parsers.h"

const char * myname = "";
static char * external_filter = "";
static pid_t external_filter_pid = -1;
extern pid_t child;
int restart = 0;

void interrupt(int)
{
    signal(SIGINT, interrupt);
    if ( verbose )
        outputs(2, "*** sigint ***\n", 0);
    if ( external_filter_pid > 0 )
        kill(-external_filter_pid, SIGINT);
    if ( child > 0 )
        kill(-child, SIGINT);
    exit(1);
}
void terminate(int)
{
    if ( verbose )
        outputs(2, "terminate\n", 0);
    if ( external_filter_pid > 0 )
        kill(-external_filter_pid, SIGHUP);
    if ( child > 0 )
        kill(-child, SIGHUP);
    exit(1);
}

// pass everything on stdin of an external program.
static int external(int fd, char * line, int len)
{
    if ( !*external_filter )
        return NEXT; // no external filter given

    static int hh[2] = {-1, -1};

    if ( external_filter_pid < 0 )
    {
        if ( pipe(hh) < 0 )
            return outputs(2, myname, ": pipe:", strerror(errno), "\n", 0),
                FAIL;

        external_filter_pid = fork();
        if ( external_filter_pid < 0 )
            return outputs(2, myname, ": ", strerror(errno), "\n", 0), FAIL;

        if ( 0 == external_filter_pid ) // child (filter process)
        {
            close(hh[1]);
            dup2(hh[0], 0);
            close(hh[0]);

            static char * args[3] = {0};
			args[0] = "sh";
            args[1] = "-c";
            args[2] = external_filter;
            execvp(args[0], args);
            outputs(2, myname, ": exec: ", external_filter,
                    ": ", strerror(errno), "\n", 0);
            exit(99);
        }
        // parent
        close(hh[0]);
    }
    write(hh[1], line, strlen(line)); // send the line to the filter
    return DONE; // filter is completely responsible for output
}

// default line handler (just prints the line)
static int print_default(int fd, char * line, int len)
{
    if ( len ) write(1, line, len);
    return STOP; // i am the last.
}

static hook_t hooks[] = {
    { &syslogmsgs    },
    { &external      },
    { &print_default }
};

// stolen from linux/lib/vsprintf.c
#define SPECIAL	32		/* 0x */

#define do_div(n,base)						\
({											\
	int _res;								\
	_res = ((unsigned long) (n)) % (unsigned) (base);	\
	(n) = ((unsigned long) (n)) / (unsigned) (base);	\
	_res;									\
})

static char * number(char * buf, char * end, long num, int base)
{
	char c,sign,tmp[66];
	const char *digits = "0123456789abcdef";
	int i;

	if (base < 2 || base > 16)
		return 0;
	c = ' ';
	sign = 0;
	if (num < 0) {
		sign = '-';
		num = -num;
	}
	i = 0;
	if (num == 0)
		tmp[i++]='0';
	else while (num != 0)
		tmp[i++] = digits[do_div(num,base)];
	if (sign) {
		if (buf <= end)
			*buf = sign;
		++buf;
	}
	while (i-- > 0) {
		if (buf <= end)
			*buf = tmp[i];
		++buf;
	}
	return buf;
}
int main(int, char **argv)
{
    verbose = 0;
    int merge = 1;
    char * argv0 = *argv;

    for ( char ** p = ++argv; *p; ++p )
    {
        if ( '-' != **p )
            break;
        for ( char * pc = p[0] + 1; *pc; ++pc )
            switch ( *pc )
            {
            case 's': // separate stdout/stderr
                merge = 0;
                if ( p + 1 > argv ) argv = p + 1;
                break;
            case 'v': // verbose (show the commands after fork
                verbose = 1;
                if ( p + 1 > argv ) argv = p + 1;
                break;
            case 'c':
                if ( *p[1] )
                {
                    ++p;
                    external_filter = *p;
                    if ( p + 1 > argv ) argv = p + 1;
                    goto _next_param;
                }
                break;
			case 'R':
				restart = 1;
                if ( p + 1 > argv ) argv = p + 1;
				break;
            case '-': // end of params
                argv = p + 1;
            default:  // unknown parameter
                goto _start;
            }
    _next_param:
        ;
    }
_start:
    if ( strrchr(argv0, '/') )
        myname = strrchr(argv0, '/') + 1;
    if ( verbose )
    {
        outputs(1, myname, ": verbose mode\n", 0);
        if ( !merge )
            outputs(1,myname, ": separating stdout and stderr for child\n",0);
        if ( *argv )
        {
            outputs(1, myname, ": parameters(concatenated): ", 0);
            output(1, argv, -1);
            outputs(1, "\n", 0);
        }
    }
    if ( !*argv )
    {
        outputs(2,
                myname, ": no command given\n",
                myname, ": an output preprocessor\n",
                myname, " [-sv -c <filter command> --] command ...\n"
                " -s  separate stdout and stderr into different streams\n"
                " -v  verbose\n"
                " -c  set the filter command (it will be responsible"
                " for all unhandled output);\n"
                " -R  restart the command after it has finished\n"
                "     the command is executed through your $SHELL.\n"
                " \"--\" can be used to separate the command beginning\n"
                "        with dash from the parameters. Everything after\n"
                "        the dashes will be passed to the execvp(3).\n",
                0);
        exit(1);
    }
    signal(SIGINT, interrupt);
    signal(SIGTERM, terminate);
    signal(SIGPIPE, terminate);
    signal(SIGHUP, terminate);
	int status = 0;
	do
	{
		status = ptyrun(argv, hooks, sizeof(hooks)/sizeof(*hooks), merge);
		if ( verbose && restart )
		{
			char num[32];
			*number(num, num+32, status, 10) = '\0';
			outputs(2, myname, ": ", *argv, " finished with ", num, "\n", 0);
		}
	}
	while ( restart && 255 != status ); // spawn_pty return 255 if exec fail

    // place exit code handling here
    if ( verbose )
        outputs(2, myname, ": the end.\n", 0);

    return status;
}

