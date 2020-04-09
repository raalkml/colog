#
# Colorizing log output
#
target = colog
sources :=        \
    main.c        \
    syslogmsgs.c  \
    ptyrun.c      \
    output.c      \
    match.c       \

odir := O
ofiles = $(patsubst %.cc,$(odir)/%.o,$(patsubst %.c,$(odir)/%.o,$(sources)))

CC := gcc
CXX := g++
CFLAGS := -Wall -ggdb -O3
CXXFLAGS := -Wall -ggdb -O3
CPPFLAGS := -DTEST_CODE -Darch_$(shell uname -s)
LDFLAGS := -Wall -ggdb -O3
LDLIBS :=
PKG_CFLAGS =
PKG_LIBS =

build: $(target)

$(target): $(ofiles)
	$(LINK.o) $^ $(LOADLIBES) $(PKG_LIBS) $(LDLIBS) $(LOADLIBES) -o $@

$(odir)/%.o: %.c
	@mkdir -p '$(odir)'
	$(COMPILE.c) $(OUTPUT_OPTION) $(PKG_CFLAGS) $(CPPFLAGS) $(CFLAGS) $<

$(odir)/%.o: %.cc
	@mkdir -p '$(odir)'
	$(COMPILE.cc) $(OUTPUT_OPTION) $(PKG_CFLAGS) $(CPPFLAGS) $(CXXFLAGS) $<

rebuild: clean
	$(MAKE) build

depclean:
	rm -f $(odir)/*.d
clean:
	rm -f $(odir)/*.o $(target)
distclean: clean depclean

install:
	install -m 0755 $(target)

tags:
	ctags $(sources) *.h *.hh *.hpp

$(odir)/%.d: %.c
	@mkdir -p '$(odir)'
	$(CC) -MM -o $@ $< $(CPPFLAGS)

$(odir)/%.d: %.cc
	@mkdir -p '$(odir)'
	$(CC) -MM -o $@ $< $(CPPFLAGS)

.PHONY: tags build rebuild depclean clean distclean install

ifneq (clean,$(findstring clean,$(MAKECMDGOALS)))
-include $(patsubst %.cc,$(odir)/%.d,$(patsubst %.c,$(odir)/%.d,$(sources)))
endif

.PHONY: test
test: $(target)
	$(abspath $<) cat test/*
#	@$(ARCH)/$(PROGNAME) -vs sh -c 'echo Test1& echo >&2 Test2& echo Test3& wait'

#install: all
#	-test -d $(HOME)/bin/$(ARCH) && \
#	cp $(ARCH)/$(PROGNAME) $(HOME)/bin/$(ARCH)/$(PROGNAME)

