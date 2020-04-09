/*
 * syslog style output colorification
 */
#include <string.h>
#include "parsers.h"

int syslogmsgs(int fd, char * line, int len)
{
	static void *syslogdate;
	char *p;
	int pos, length;

	if (!syslogdate)
		syslogdate = match_prepare("^... +[0-9]+ [0-9]+:[0-9]+:[0-9]+ ");
	if (!syslogdate)
		goto skip;
	if (match_find(syslogdate, line, &pos, &length) < 0)
		goto skip;
	if ( len > 0 && line[len-1] == '\n' )
		--len;
	// date
	output(fd, esc_ bold ";" yellow _esc, -1, line, length, 0, 0);
	line += length;
	len -= length;
	p = memchr(line, '\x20', len);
	if (p) {
		int ll = p - line;
		output(fd, esc_ cyan _esc, -1,
		       line, ll,
		       esc_ bold_off _esc, -1,
		       0,0);
		line += ll;
		len -= ll;
		// syslog tag, possibly with pid in square brackets
		p = memchr(line, ':', len);
		if (p) {
			char *p1, *p2;
			++p;
			--len;
			ll = p - line;
			p1 = memchr(line, '[', ll);
			p2 = memchr(line, ']', ll);
			if (p1 && p2 && p2 > p1) {
				// output(fd, esc_ magenta _esc, -1,
				// 		line, p1 - line,
				// 		esc_ bold _esc, -1,
				// 		p1, p2 - p1 + 1,
				// 		esc_ bold_off _esc, -1,
				// 		p2+1, p - p2,
				// 		esc_ cyan _esc, -1,
				// 		p, len - ll,
				// 		0,0);
				output(fd, esc_ white _esc, -1,
				       line, p1 - line,
				       esc_ bold _esc, -1, p1, 1, esc_ bold_off _esc, -1,
				       p1 + 1, p2 - p1 - 1,
				       esc_ bold _esc, -1, p2, 1, esc_ bold_off _esc, -1,
				       p2+1, p - p2-1,
				       esc_ cyan _esc, -1,
				       0,0);
			} else {
				output(fd, esc_ white _esc, -1,
				       line, ll,
				       esc_ cyan _esc, -1,
				       0,0);
			}
			line = p;
			len -= ll - 1;
		}
		p = strstr(line, "alarm");
		if (p) {
			output(fd, line, p - line,
			       esc_ bold ";" red _esc, -1,
			       p, 5,
			       esc_ bold_off ";" cyan _esc, -1,
			       0,0);
			len -= p - line + 5;
			line = p + 5;
		}
		output(fd, line, len, 0,0);
	} else {
		output(fd, esc_ bold_off ";" cyan _esc, -1, line, len, 0,0);
	}
	output(fd, esc_ normal _esc "\n", -1, 0,0);
	return DONE;
skip:
	return NEXT;
}
