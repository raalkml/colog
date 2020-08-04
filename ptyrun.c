#if defined(arch_linux) || defined(arch_Linux)
# define _GNU_SOURCE
# define _XOPEN_SOURCE
# define _XOPEN_SOURCE_EXTENDED
# include <sys/ioctl.h>
# include <sys/termios.h>
#ifdef HAVE_STREAMS
# include <sys/stropts.h>
#endif
# include <stdlib.h>

#elif defined(arch_sparcOS5) || defined(arch_gccsparcOS5)
# define _XOPEN_SOURCE_EXTENDED
# include <stropts.h>
# include <sys/termios.h>

#elif defined(arch_hpux10) || defined(arch_gcchpux)
# define _XOPEN_SOURCE_EXTENDED
extern "C" {
# include <stropts.h>
};
# include <sys/termios.h>

#elif defined(arch_rs6000) || defined(arch_gccrs6000)
# include <stropts.h>

#else
#warning "PTY maybe unsupported, trying HP-UX-like method"
#include <stropts.h>
#endif

#include <unistd.h>

#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <string.h>
#include <poll.h>
#include <sys/poll.h>
#include <errno.h>
#include <signal.h>
#include <sys/wait.h>
#include <sys/time.h>

#include "output.h"
#include "ptyrun.h"

int verbose = 1;

void show_params(char * const * argv, int fd)
{
    for (char * const * p = argv; *p; ++p)
    {
        if ( p == argv )
            write(fd, "'", 1);
        else
            write(fd, " '", 2);
        write(fd, *p, strlen(*p));
        write(fd, "'", 1);
    }
    write(fd, "\n", 1);
}

int spawn_pty(char * const * argv, pid_t * pid, int * errfd)
{
    char * ttydev = 0;
    int master = -1, tty = -1, errtty = -1;
    
    if ( verbose )
        show_params(argv, 1);
    
    // will fallback to pipes if allocation of pty is failed
    int nopty = 0;

#if defined(arch_sparcOS5) || defined(arch_linux) || defined(arch_gccsparcOS5)
    // open the STREAMS, clone device /dev/ptmx (master pty)
    master = open("/dev/ptmx", O_RDWR);
    
	if ( master < 0 )
        nopty = 1;
    else
    {
	    grantpt(master);  // change slave permissions
	    unlockpt(master); // unlock slave
	    ttydev = strdup(ptsname(master)); // get slave's name
        
        if ( !ttydev  )
            nopty = 1;
	}
#else // HP-UX and others
    const char *c1, *c2;
    char pty_name[] = "/dev/pty??";
    char tty_name[] = "/dev/tty??";
    int len = strlen(tty_name);
    
    ttydev = tty_name;
    
    for (c1 = "pqrstuvwxyz"; *c1; c1++)
    {
        pty_name[len - 2] = tty_name[len - 2] = *c1;
        
        for (c2 = "0123456789abcdef"; *c2; c2++)
        {
            pty_name[len - 1] = tty_name[len - 1] = *c2;
            
            if ( (master = open(pty_name, O_RDWR)) < 0 )
                continue;
            
            if (access(tty_name, R_OK | W_OK) == 0)
                goto _found_pty;
            
            close(master);
        }
    }
    nopty = 1;
_found_pty:
#endif
    if ( nopty ) // fall back to pipe
    {
        int hh[2];
        
        if ( pipe(hh) < 0 )
            return -1; // no chance
        
        master = hh[0];
        tty = hh[1];
    }
    if ( errfd ) // expecting error output separate
    {
        int hh[2];
        
        if ( pipe(hh) < 0 )
        {
            close(master);
            if ( nopty )
                close(tty);
            return -1;
        }
        *errfd = hh[0];
        errtty = hh[1];
    }
    if ( (*pid = fork()) < 0 )
    {
        close(master);
        if ( nopty )
            close(tty);
        if ( errfd )
        {
            close(*errfd);
            close(errtty);
        }
        return -1;
    }
    if ( 0 == *pid )
    {
        // child
        close(master); // close master pty (not used)
        if ( errfd )
            close(*errfd);
        
        setsid();
        if ( !nopty )
        {
            tty = open(ttydev, O_WRONLY);
            
            if ( tty < 0 )
                exit(ENOTTY);
#ifdef HAVE_STREAMS
            ioctl(tty, I_PUSH, "ptem");
#if !(defined(arch_rs6000) || defined(arch_gccrs6000))
            // aix hangs here
            ioctl(tty, I_PUSH, "ldterm");
#endif
            ioctl(tty, I_PUSH, "ttcompat");
#endif
        }
        // redirect stdin into /dev/null
        int null = open("/dev/null", O_RDONLY);
        dup2(null, 0);
        close(null);
        // redirect output streams into allocated pty
        dup2(tty, 1);
        if ( errfd )
        {
            dup2(errtty, 2);
            close(errtty);
        }
        else
        {
            dup2(tty, 2);
        }
        close(tty);
#ifdef HAVE_SETENV
        setenv("COLUMNS", "80", 1);
        setenv("LINES",   "25", 1);
#else
        putenv("COLUMNS=80");
        putenv("LINES=25");
#endif
        
        execvp(*argv, argv);
        
        if ( verbose )
            outputs(2, *argv, ": ", strerror(errno), "\n", 0);
        
        exit(255);
    }
    // parent process
    if ( nopty )
        close(tty);
    if ( errfd )
        close(errtty);
#if defined(arch_sparcOS5) || defined(arch_linux)
    free(ttydev);
#endif
    // set non-blocking mode
    int flags = fcntl(master, F_GETFL);
    
    if ( flags != -1 )
        fcntl(master, F_SETFL, flags|O_NONBLOCK|FD_CLOEXEC);
    if ( errfd )
    {
        flags = fcntl(*errfd, F_GETFL);
        
        if ( flags != -1 )
            fcntl(*errfd, F_SETFL, flags|O_NONBLOCK|FD_CLOEXEC);
    }
    return master;
}


struct child_stream
{
    char * base, * line, * p, * eol;
    int size;
    int hangup;
};

static void child_stream_init(struct child_stream *strm)
{
    *strm = (struct child_stream){};
}

static void child_stream_destroy(struct child_stream *strm)
{
    free(strm->base);
}

static void child_stream_checkbuf(struct child_stream *strm)
{
    if ( strm->p - strm->base + 1 > strm->size )
    {
        int offs = strm->p - strm->base;
        strm->size += 512;
        strm->base = (char*)realloc(strm->base, strm->size);
        strm->eol = strm->p = strm->base + offs;
    }
}

static int child_stream_put(struct child_stream *strm, char c)
{
    child_stream_checkbuf(strm);
    *strm->p++ = c;
    strm->eol = strm->p;
    return c;
}

static int child_stream_readch(struct child_stream *strm, int fd)
{
    char ch;
    int rc = read(fd, &ch, 1);
    if ( 1 == rc )
        child_stream_put(strm, ch);
    return rc;
}

static char child_stream_last(const struct child_stream *strm)
{
    return strm->p > strm->base ? strm->p[-1]: '\0';
}

static void child_stream_write_down(struct child_stream *strm)
{
    child_stream_put(strm, 0); // null-terminator for the line
    strm->line = strm->base;
    strm->p = strm->base;
}

static void child_stream_reset_line(struct child_stream *strm)
{
    strm->line = 0;
    strm->eol = strm->p;
}

#if defined(arch_hpux10) || defined(arch_gcchpux)
// there is no clear statement in which cases poll(2) on HP-UX
// sends POLLIN and in which POLLRDNORM(is the same, but other value).
// just catch both.
#define POLLMASK (POLLIN|POLLRDNORM)
#else
#define POLLMASK POLLIN
#endif

static int child_stream_poll_child(struct child_stream *strm, struct pollfd *pfd)
{
    int rc = 0;
    
    if ( pfd->revents & POLLMASK )
    {
        // write(2, "+", 1);
        if ( (rc = child_stream_readch(strm, pfd->fd)) <= 0 )
        {
            int err = errno;
#if defined(arch_hpux10)
            if ( !isatty(pfd->fd) && 0 == rc && 0 == err )
            {
                // hp-ux: read(2) return 0 and sets errno to zero if
                // the _pipe_ where it have been reading from is closed.
                // It is all right. But the damn thing doesn't do
                // anything if the channel is pty! One must check
                // if the child still around.
                child_stream_write_down(strm);
                return -1;
            }
#elif defined(arch_sparcOS5) || defined(__sun__)
            if ( 0 == rc && 0 == err )
                // sun: the incorrect behaviour of the os is to return
                // 0 in read(2) and 0 in errno if there is no input
                // on non-blocking channel
                return 0;
#endif
            if ( rc < 0 && EAGAIN == err )
                // linux, hp-ux: posix approved behaviour for
                // non-blocking channel having no input.
                return 0;
            
            // linux: error reading from channel
            // hp-ux, sun: error or eof
            child_stream_write_down(strm);
            return -1;
        }
        if ( strm->hangup )
            return -1;
        if ( '\n' == child_stream_last(strm) )
            child_stream_write_down(strm);
        return 0;
    }
    if ( pfd->revents & (POLLHUP|POLLERR|POLLNVAL) )
    {
        // linux: posixly correct, these pollhup is sent to the process
        // by poll(2) if the polled channel gets closed.
        // sun: POLLHUP is NOT sent for ptys.
        if ( strm->p > strm->base )
            child_stream_write_down(strm);
        strm->hangup = 1;
        return -1;
    }
    return 0;
}

int do_buffer(struct child_stream *buf, struct pollfd *fds,
              struct hook *hooks, int hcount, int id)
{
    int read_ok = child_stream_poll_child(buf, fds);
    
    if ( buf->line )
    {
        int llen = buf->eol - buf->line - 1 /* \0 */;
        if ( llen >= 2 && !strcmp(buf->eol - 3, "\r\n") )
        {
            buf->eol[-3] = '\n';
            buf->eol[-2] = '\0';
            --llen;
        }
        // line ready. Run the hooks over it.
        for ( int i = 0; i < hcount; ++i )
        {
            if ( (*hooks[i].proc)(id, buf->line, llen) == 0 )
                break;
        }
        child_stream_reset_line(buf);
    }
    if ( read_ok < 0 )
        return -1;
    return 0;
}

pid_t child = 0;

int ptyrun(char* argv[], struct hook * hooks, int hcount, int merge)
{
    struct pollfd fds[2];
#if defined(arch_rs6000) || defined(arch_gccrs6000)
    fds[0].fd = spawn_pty(argv, &child, merge ? 0: (int*)&fds[1].fd);
#else
    fds[0].fd = spawn_pty(argv, &child, merge ? 0: &fds[1].fd);
#endif

#if !defined(arch_hpux10)
    fds[1].events = fds[0].events = POLLIN|POLLHUP|POLLRDNORM;
#else
    fds[1].events = fds[0].events = POLLIN;
#endif
    int status = -1;
    
    if ( fds[0].fd < 0 )
        return -1;
    
    struct child_stream buf, ebuf;
    int rc = -1, wok;
#if defined(arch_rs6000) || defined(arch_gccrs6000)
    int timeout = 300;
#else
    int timeout = 1000;
#endif
    wok = 0;
    struct pollfd * pfd = fds;
    int cfd = merge ? 1: 2;

    child_stream_init(&buf);
    child_stream_init(&ebuf);
_restart:
    while ( cfd > 0 && (rc = poll(pfd, cfd, timeout)) >= 0 )
    {
        if ( 0 == merge && cfd > 1 &&
             do_buffer(&ebuf, &fds[1], hooks, hcount, 2) )
        {
            --cfd;
            
            if ( verbose )
                outputf(1, "[%d] %d closed stderr\n", getpid(), child);
        }
        if ( do_buffer(&buf, &fds[0], hooks, hcount, 1) )
        {
            ++pfd;
            --cfd;
            
            if ( verbose )
                outputf(1, "[%d] %d closed stdout\n", getpid(), child);
        }
        if ( cfd && rc )
            // If there is at least one handle, and it wasn't timed out.
            continue;
        
        if ( (rc = waitpid(child, &wok, cfd ? WNOHANG: 0)) == child )
        {
            if ( WIFEXITED(wok) )
            {
                status = WEXITSTATUS(wok);
                if ( verbose )
                    outputf(1, "[%d] %d exited: %d\n", getpid(), child, status);
            }
            if ( WIFSIGNALED(wok) )
            {
                if ( verbose )
                    outputf(1, "[%d] %d signaled\n", getpid(), child);
                // killed by a signal (ours or someones else)
                status = -1;
            }
            if ( WIFEXITED(wok) || WIFSIGNALED(wok) )
                break;
        }
    }
    if ( cfd && rc < 0 && EINTR == errno ) // auto-restart some signals.
        goto _restart;
    close(fds[0].fd); // close master pty
    if ( !merge )
        close(fds[1].fd); // close error pipe
    child_stream_destroy(&ebuf);
    child_stream_destroy(&buf);
    return status;
}
