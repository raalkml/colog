// portable pty-preparser for wide range of compilers (gcc,HPaCC,SunCC,IBMxlC)
#define _GNU_SOURCE
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <regex.h>

#include "match.h"

void * match_prepare(const char * expr)
{
    regex_t * re = (regex_t *)malloc(sizeof(regex_t));
    if ( regcomp(re, expr, REG_EXTENDED|REG_ICASE) )
    {
        free(re);
        return 0;
    }
    return re;
}

// return -1, if not found, *pos otherwise
int match_find(void * pre, const char * text, int *pos, int * len)
{
    if ( len )
        *len = 0;
    if ( pos )
        *pos = 0;
    regex_t * re = (regex_t *)pre;
//    regmatch_t * matches = (regmatch_t *)malloc((re->no_sub + 1) *
//                                                sizeof(regmatch_t));
    regmatch_t match;
    if ( regexec((regex_t*)re, text, 1, &match, 0) )
        return -1;
    if ( len )
        *len = match.rm_eo - match.rm_so;
    if ( pos )
        *pos = match.rm_so;
    return match.rm_so;
}

// return strlen(str2) if str1 begins with str2, or 0
int begins(const char * str1, const char * str2)
{
    int l2 = strlen(str2);
    return 0 == strncmp(str1, str2, l2) ? l2: 0;
}

// return strlen(str2) if str1 ends with str2, or 0
int ends(const char * str1, const char * str2)
{
    int l1 = strlen(str1);
    int l2 = strlen(str2);
    if ( l1 < l2 )
        return 0;
    return 0 == strncmp(str1 + l1 - l2, str2, l2) ? l2: 0;
}

// test if the str1 is a pathname. Pathname is a sequence of characters
// looking like an absolute unix path("/opt/compilers/gcc-2.96.x/bin/cc"),
// or just a word from beginning of str1 until first character from terms
// is seen.
// Return matched length, or 0. If terms is not 0, it is used as a
// termination sequence of the possible name. '\0' is always a
// terminator. maxlen == -1 makes maxlen to strlen(str1).
int is_pathname(const char * str1, int maxlen, const char * terms)
{
    if ( maxlen < 0 )
        maxlen = strlen(str1);
    if ( !terms )
        // colon is a separator as it is often used by
        // gmake and ld after the name.
        terms = " \t\r\n:";

    if ( maxlen &&
         !('/' == *str1 ||
           (*str1 >= 'A' && *str1 <= 'Z') ||
           (*str1 >= 'a' && *str1 <= 'z') ||
           !strchr(terms, *str1)) )
        return 0;

    int result = 0;
    for ( ; maxlen-- && ('/' == *str1 || !strchr(terms, *str1)); ++str1 )
        ++result;
    return result;
}


