// text lookup routines
#ifndef __match_h__
#define __match_h__ 1

/* re */ void * match_prepare(const char * expr);

// return -1, if not found, *pos otherwise
int match_find(void * re, const char * text, int * pos, int * len);

// return strlen(str2) if str1 begins with str2, or 0
int begins(const char * str1, const char * str2);
// return strlen(str2) if str1 ends with str2, or 0
int ends(const char * str1, const char * str2);

// test if the str1 is a pathname. Pathname is a sequence of characters
// looking like an absolute unix path("/opt/compilers/gcc-2.96.x/bin/cc"),
// or just a word from beginning of str1 until first character from terms
// is seen. The first character must be [a-zA-Z/].
// Return matched length, or 0. If terms is not 0, it is used as a
// termination sequence of the possible name. '\0' is always a
// terminator. maxlen == -1 makes maxlen to strlen(str1).
int is_pathname(const char * str1, int maxlen = -1, const char * terms = 0);

#endif
#if UNDEF
struct textblk
{
	const char * p;
	int l;
	struct textblk * next;
};
struct textinfo
{
	const char * text;
	int size;
	struct textblk * head, * tail;

	textinfo(const char * txt, int len);
	textblk * mark(int pos, int len, const char *);
	textblk * mark(const char * pfx, const char * rx, const char * sfx);
	int mark_all(const char * pfx, const char * rx, const char * sfx);
};


int xxx(char * line)
{
	textinfo ti(line, strlen(line));
	ti.mark(0, ti.size, DEFAULT_COLOR);
	ti.mark(esc_ cyan _esc,
			"^[A-Z][a-z][a-z] +[0-9]+.*:[0-9][0-9]",
			DEFAULT_COLOR);
	ti.mark_all(esc_ bold _esc,
				"[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+",
				esc_ bold_off _esc);
	ti.write(fd);
}
#endif
