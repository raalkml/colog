#ifndef __escape_seq_h__
#define __escape_seq_h__ "xterm"

#define esc_ "\x1b["
// separate through ';'
#define _esc "m"

#define normal         "0"
#define bold           "1"
#define bold_off       "22"
#define underline      "4"
#define underline_off  "24"
#define blink          "5"
#define blink_off      "25"
#define inverse        "7"
#define inverse_off    "27"

#define      black "30"
#define   black_bg "40"
#define        red "31"
#define     red_bg "41"
#define      green "32"
#define   green_bg "42"
#define     yellow "33"
#define  yellow_bg "43"
#define       blue "34"
#define    blue_bg "44"
#define    magenta "35"
#define magenta_bg "45"
#define       cyan "36"
#define    cyan_bg "46"
#define      white "37"
#define   white_bg "47"
#define    _default "37"
#define _default_bg "47"

// xterm

#define xterm_title_ "\e]2;"
#define _xterm_esc "\a"

#endif
