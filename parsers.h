// portable pty-preparser for wide range of compilers (gcc,HPaCC,SunCC,IBMxlC)
#ifndef __parsers_h__
#define __parsers_h__ 1

#include "output.h" // output routines (error() & warning() also)
#include "match.h"  // search/match routines (gnu regex)

//
// return codes
//
#define DONE 0
#define STOP 0
#define NEXT -1
#define FAIL -1

int regex_colors(int fd, char *line, int len);

#endif // __parsers_h__
