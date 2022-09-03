
#   Make instructions for moon tool

PROG=	moontool
CFILES= moontool.c
OFILES= moontool.o
LIBS=	-lm -lsuntool -lsunwindow -lpixrect
ICONS=	moon.icon colormoon.icon

XPROG = xmoontool
XCFILES = xmoontool.c icongeom.c
XOFILES = xmoontool.o icongeom.o
XLIBS = -lm -lXol -lXt -lX11
XPATHS = -I$$OPENWINHOME/include -L$$OPENWINHOME/lib
XICONS = moon_icon

SFILES= README moontool.1 xmoontool.1 Makefile \
	$(CFILES) $(XCFILES) $(ICONS) $(XICONS)

#CFLAGS= -O $(XPATHS)
CFLAGS = -g $(XPATHS)
#LDFLAGS= -O
LDFLAGS= -g

all:	$(PROG) $(XPROG)

$(PROG): $(OFILES)
	$(CC) $(LDFLAGS) -o $@ $(OFILES) $(LIBS)
	rm -f core
	strip moontool

$(XPROG): $(XOFILES)
	$(CC) $(LDFLAGS) -o $@ $(XOFILES) $(XLIBS)
	rm -f core
	strip xmoontool

moontool.o: $(ICONS) 

clean:
	rm -f $(PROG) $(XPROG)
	rm -f *.o *.bak
	rm -f core cscope.out *.shar

manpage:
	nroff -man moontool.1 | more

xmanpage:
	nroff -man xmoontool.1 | more

lint:
	lint $(CFILES) $(LIBS)

shar:
	shar -b -v $(SFILES) >$(PROG).shar
	rm -f $(PROG).shar.gz
	gzip $(PROG).shar
