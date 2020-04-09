#define _GNU_SOURCE
#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <regex.h>
#include "parsers.h"

#define COLORMAPSIZE 100

struct regex_color
{
	int fd;
	const char *expr;
	const char *clr[COLORMAPSIZE];
	regex_t re;
};

static struct regex_color colors[] = {
	{ /* tail(1) with multiple files */
		STDIN_FILENO,
		"^==> (.*) <==$",
		{
			esc_ magenta _esc,
			esc_ _default _esc,
		}
	},
	{ /* postfix deferred */
		STDIN_FILENO,
		"^(...[ \t]*[0-9]+[ \t]+..:..:.. )([^ \t]+ )(postfix/smtp)(\\[[0-9]+\\])(:)(.*, status=deferred )(.*)",
		{
			esc_ cyan _esc,
			esc_ yellow _esc,
			esc_ cyan _esc,
			esc_ bold ";" green _esc,
			esc_ bold_off ";" green _esc,
			esc_ bold ";" cyan _esc,
			esc_ bold_off ";" cyan _esc,
			esc_ bold_off ";" red _esc,
		}
	},
	{ /* syslog facility with a pid */
		STDIN_FILENO,
		"^(...[ \t]*[0-9]+[ \t]+..:..:.. )([^ \t]+ )([^ \t]+)(\\[[0-9]+\\])(:)(.*)",
		{
			esc_ cyan _esc,
			esc_ yellow _esc,
			esc_ cyan _esc,
			esc_ bold ";" green _esc,
			esc_ bold_off ";" green _esc,
			esc_ bold ";" cyan _esc,
			esc_ bold_off ";" cyan _esc,
		}
	},
	{ /* syslog facility without a pid */
		STDIN_FILENO,
		"^(...[ \t]*[0-9]+[ \t]+..:..:.. )([^ \t]+ )([^ \t[]+):(.*)",
		{
			esc_ cyan _esc,
			esc_ yellow _esc,
			esc_ cyan _esc,
			esc_ bold ";" green _esc,
			esc_ bold_off ";" cyan _esc,
		}
	},
};

extern int verbose;
static regmatch_t m[COLORMAPSIZE];
static struct regex_color *rc;

static void writes(int fd, const char *s)
{
	if (s)
		(void)write(fd, s, strlen(s));
}
static void writen(int fd, const char *s, int n)
{
	if (s && n)
		(void)write(fd, s, n);
}

void recursive_output(int fd, char *line, int sub, int psub)
{
	int this = sub;
	int so = m[this].rm_so, eo = m[this].rm_eo;

	if (so == eo)
		return;
	while (sub < rc->re.re_nsub) {
		++sub;
		if (m[sub].rm_so == m[sub].rm_eo)
			continue;
		if (m[sub].rm_so >= eo)
			break;
		writes(fd, rc->clr[this]);
		writen(fd, line + so, m[sub].rm_so - so);
		recursive_output(fd, line, sub, this);
		so = m[sub].rm_eo;
	}
	if (so < eo) {
		writes(fd, rc->clr[this]);
		writen(fd, line + so, eo - so);
	}
	writes(fd, psub == -1 ? esc_ normal _esc : rc->clr[psub]);
}

int regex_colors(int fd, char *line, int len)
{
	static const char IGNORE_ITEM[1];
	unsigned int i;

	if ( len > 0 && line[len-1] == '\n' )
		--len;
	for (i = 0; i < sizeof(colors) / sizeof(*colors); ++i) {
		rc = &colors[i];
		if (!rc->expr)
			break;
		if (rc->expr == IGNORE_ITEM)
			continue;
		if (!rc->re.buffer) {
			int err = regcomp(&rc->re, rc->expr, REG_EXTENDED | REG_ICASE);
			if (err) {
				error(STDERR_FILENO, "regex error:", -1, rc->expr, -1);
				rc->expr = IGNORE_ITEM;
				continue;
			}
			if (verbose)
				fprintf(stderr, "%s: re_nsub=%zu\n", rc->expr, rc->re.re_nsub);
		}
		m[0].rm_so = 0;
		m[0].rm_eo = len;
		if (regexec(&rc->re, line, rc->re.re_nsub + 1, m, REG_STARTEND) == 0) {
			writes(fd, esc_ normal _esc);
			writen(fd, line, m[0].rm_so);
			recursive_output(fd, line, 0, -1);
			writes(fd, esc_ normal _esc);
			writen(fd, line + m[0].rm_eo, len - m[0].rm_eo);
			writen(fd, "\n", 1);
			return DONE;
		}
	}
	return NEXT;
}

