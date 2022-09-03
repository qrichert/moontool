/*

			A Moon for OpenWindows

			     Release 3.1

    Designed and implemented by John Walker in December 1987,
    Revised and updated in February of 1988.
    Revised and updated again in June of 1988 by Ron Hitchens.
    Revised and updated yet again in July/August of 1989 by Ron Hitchens.
    Converted to OpenWindows in December of 1991 by John Walker.

    This  program  is  an OpenWindows tool which displays, as the icon
    for a closed window, the current phase of the Moon.  A subtitle in
    the  icon  gives the age of the Moon in days and hours.  If called
    with the "-t" switch, it rapidly increments forward  through  time
    to display the cycle of phases.

    If you  open  the  window,	additional  information  is  displayed
    regarding  the  Moon.   The  information  is generally accurate to
    within ten minutes.

    The algorithms used in this program to calculate the positions Sun
    and  Moon  as seen from the Earth are given in the book "Practical
    Astronomy With Your Calculator"  by  Peter  Duffett-Smith,  Second
    Edition,  Cambridge  University  Press,  1981.   Ignore  the  word
    "Calculator" in the title;  this  is  an  essential  reference  if
    you're   interested   in   developing  software  which  calculates
    planetary positions, orbits, eclipses, and the  like.   If  you're
    interested	in  pursuing such programming, you should also obtain:

    "Astronomical Formulae  for  Calculators"  by  Jean  Meeus,  Third
    Edition, Willmann-Bell, 1985.  A must-have.

    "Planetary  Programs  and  Tables  from  -4000 to +2800" by Pierre
    Bretagnon and Jean-Louis Simon, Willmann-Bell, 1986.  If you  want
    the  utmost  (outside of JPL) accuracy for the planets, it's here.

    "Celestial BASIC" by Eric Burgess, Revised Edition,  Sybex,  1985.
    Very cookbook oriented, and many of the algorithms are hard to dig
    out of the turgid BASIC code, but you'll probably want it anyway.

    Many  of these references can be obtained from Willmann-Bell, P.O.
    Box 35025, Richmond, VA 23235, USA.  Phone:  (804)	320-7016.   In
    addition  to  their  own  publications,  they  stock  most	of the
    standard references for mathematical and positional astronomy.

    This program was written by:

	John Walker
	http://www.fourmilab.ch/

    This  program is in the public domain: "Do what thou wilt shall be
    the whole of the law".  I'd appreciate  receiving  any  bug  fixes
    and/or  enhancements, which I'll incorporate in future versions of
    the program.  Please leave the  original  attribution  information
    intact so that credit and blame may be properly apportioned.

	History:
	--------
	June 1988	Version 2.0 posted to usenet by John Walker

	June 1988	Modified by Ron Hitchens
			     ronbo@vixen.uucp
			     ...!uunet!cs.utah.edu!caeco!vixen!ronbo
			     hitchens@cs.utexas.edu
			to produce version 2.1.
			Modified icon generation to show surface texture
			 on visible moon face.
			Added a menu to allow switching in and out of
			 test mode, for entertainment value mostly.
			Modified layout of information in open window display
			 to reduce the amount of pixels modified in each
			 update.

	July 1989	Modified further for color displays.  On a color Sun,
			 four colors will be used for the canvas and the icon.
			 Rather than just show the illuminated portion of
			 the moon, a color icon will also show the darkened
			 portion in a dark blue shade.	The text on the icon
                         will also be drawn in a nice "buff" color, since there
			 was one more color left to use.
                        Add two command line args, "-c" and "-m" to explicitly
			 specify color or monochrome mode.
			Use getopt to parse the args.
			Change the tool menu slightly to use only one item
			 for switching in and out of test mode.

	July 1989	Modified a little bit more a few days later to use 8
			 colors and an accurate grey-scale moon face created
			 by Joe Hitchens on an Amiga.
			Added The Apollo 11 Commemorative Red Dot, to show
			 where Neil and Buzz went on vacation a few years ago.
			Updated man page.

        August 1989     Received version 2.3 of John Walker's original code.
			Rolled in bug fixes to astronomical algorithms:

                         2.1  6/16/88   Bug fix.  Table of phases didn't update
					at the moment of the new moon.	Call on
                                        phasehunt  didn't  convert civil Julian
					date  to  astronomical	 Julian   date.
					Reported by Dag Bruck
					 (dag@control.lth.se).

			 2.2  N/A	(Superseded by new moon icon.)

			 2.3  6/7/89	Bug fix.  Table of phases  skipped  the
					phases	for  July  1989.  This occurred
					due  to  sloppy  maintenance   of   the
					synodic  month index in the interchange
					of information between phasehunt()  and
					meanphase().  I simplified and
					corrected  the	handling  of  the month
					index as phasehunt()  steps  along  and
					removed unneeded code from meanphase().
					Reported by Bill Randle  of  Tektronix.
					 (billr@saab.CNA.TEK.COM).

	January 1990	Ported to OpenWindows by John Walker.
                        All  of Ron Hitchens' additions which were not
			Sun-specific  were   carried   on   into   the
			OpenWindows version.

	December 1999	Minor Y2K fix.	Release 3.1.

*/

#include <stdio.h>
#include <math.h>

#include <X11/Intrinsic.h>
#include <X11/StringDefs.h>
#include <X11/Shell.h>
#include <Xol/OpenLook.h>
#include <Xol/ControlAre.h>
#include <Xol/Stub.h>
#include <Xol/StaticText.h>
#include <Xol/RectButton.h>

#define MOON_TITLE \
        "XMoontool   by John Walker   v3.1"

#include "moon_icon"

static char moon_icon_work[sizeof moon_icon_bits];

static char moon_icon_last[sizeof moon_icon_bits] = "abcd";
static char icon_label_last[20] = "";

#define CLOSED_SECS	120		/* update interval when tool closed */
#define OPEN_SECS	1		/* update interval when open */

/*			     0	   1	 2     3     4	   5	 6     7   */
#define COLOR_R 	{ 0x00, 0xe0, 0xc0, 0x90, 0xff, 0x10, 0x10, 0x10 }
#define COLOR_G 	{ 0x00, 0xf0, 0xd0, 0xa0, 0x80, 0x10, 0x10, 0x10 }
#define COLOR_B 	{ 0x00, 0xff, 0xe0, 0xb0, 0x80, 0x90, 0x70, 0x50 }


/*  Astronomical constants  */

#define epoch	    2444238.5	   /* 1980 January 0.0 */

/*  Constants defining the Sun's apparent orbit  */

#define elonge	    278.833540	   /* Ecliptic longitude of the Sun
				      at epoch 1980.0 */
#define elongp	    282.596403	   /* Ecliptic longitude of the Sun at
				      perigee */
#define eccent      0.016718       /* Eccentricity of Earth's orbit */
#define sunsmax     1.495985e8     /* Semi-major axis of Earth's orbit, km */
#define sunangsiz   0.533128       /* Sun's angular size, degrees, at
				      semi-major axis distance */

/*  Elements of the Moon's orbit, epoch 1980.0  */

#define mmlong      64.975464      /* Moon's mean lonigitude at the epoch */
#define mmlongp     349.383063	   /* Mean longitude of the perigee at the
				      epoch */
#define mlnode	    151.950429	   /* Mean longitude of the node at the
				      epoch */
#define minc        5.145396       /* Inclination of the Moon's orbit */
#define mecc        0.054900       /* Eccentricity of the Moon's orbit */
#define mangsiz     0.5181         /* Moon's angular size at distance a
				      from Earth */
#define msmax       384401.0       /* Semi-major axis of Moon's orbit in km */
#define mparallax   0.9507	   /* Parallax at distance a from Earth */
#define synmonth    29.53058868    /* Synodic month (new Moon to new Moon) */
#define lunatbase   2423436.0      /* Base date for E. W. Brown's numbered
				      series of lunations (1923 January 16) */

/*  Properties of the Earth  */

#define earthrad    6378.16	   /* Radius of Earth in kilometres */


#define PI 3.14159265358979323846  /* Assume not near black hole nor in
				      Tennessee */

/*  Handy mathematical functions  */

#define sgn(x) (((x) < 0) ? -1 : ((x) > 0 ? 1 : 0))	  /* Extract sign */
#define abs(x) ((x) < 0 ? (-(x)) : (x)) 		  /* Absolute val */
#define fixangle(a) ((a) - 360.0 * (floor((a) / 360.0)))  /* Fix angle	  */
#define torad(d) ((d) * (PI / 180.0))			  /* Deg->Rad	  */
#define todeg(d) ((d) * (180.0 / PI))			  /* Rad->Deg	  */
#define dsin(x) (sin(torad((x))))			  /* Sin from deg */
#define dcos(x) (cos(torad((x))))			  /* Cos from deg */

static int testmode = FALSE;	      /* Rapid warp through time for debugging */
static double faketime = 0.0;	      /* Time increment for test mode */
static int color_mode = FALSE;	      /* indicates color/mono mode */
static Boolean iconic = TRUE;	      /* Iconic ? */
static double nptime = 0.0;	      /* Next new moon time */

static char *moname[] = {
    "January", "February", "March", "April", "May",
    "June", "July", "August", "September",
    "October", "November", "December"
};

static char *labels[] = {
    "Julian date:",
    "Universal time:",
    "Local time:",
    "",
    "Age of moon:",
    "Moon phase:",
    "Moon's distance:",
    "Moon subtends:",
    "",
    "Sun's distance:",
    "Sun subtends:",
    "",
    "Last new moon:",
    "First quarter:",
    "Full moon:",
    "Last quarter:",
    "Next new moon:"
};
#define Nlabels ((sizeof labels) / sizeof(char *))

static Widget wlabel[Nlabels];	      /* Widgets for displaying values */
static Widget loonie[2];	      /* Lunation display widgets */
static Widget toplevel; 	      /* Top level application shell */
static Widget canvas;		      /* Canvas for drawing moon image */
static Pixmap p, p1;		      /* Pixmaps swapped into icon */
static XGCValues gcv;		      /* Dummy values for graphics context */
static GC gc, gc1;		      /* Graphics context */
static XtAppContext app;	      /* Application context */
static XtIntervalId tid = NULL;       /* Active timeout interval ID */

static char olabel[Nlabels][60];      /* Old label values */
static char luabel[2][60];	      /* Old lunation values */

/*  Forward functions  */

static double jtime(), phase();
static void phasehunt(), fmt_phase_time();
static void ringgg(), jyear(), jhms();

extern long time();
extern char *icongeom();

/*  EXPOSE  --	Graphics area repaint procedure.  */

static void expose(w, xevent, region)
  Widget w;
  XEvent *xevent;
  Region *region;
{
    moon_icon_last[0] = ~moon_icon_work[0];   /* Force screen refresh */
    ringgg(toplevel, NULL);
}

/*  RESIZE  --	Graphics area resize procedure.  */

static void resize(w)
  Widget w;
{
    if (XtIsRealized(w)) {
	XClearArea(XtDisplay(w), XtWindow(w), 0, 0, 0, 0, TRUE);
    }
}

/*  EPROC  --  Intercept expose and UnmapNotify events to discover when
	       the window is iconised and restored to full size.  There
               must be a better way to do this, but I'll be darned if I
	       can figger it out!  */

static void eproc(w, client_data, event, ctd)
  Widget w;
  XtPointer client_data;
  XEvent *event;
  Boolean *ctd;
{
    Boolean liconic = iconic;

    if (event->type == Expose || event->type == MapNotify) {
	iconic = FALSE;
    } else if (event->type == UnmapNotify) {
	iconic = TRUE;
    }

    /* If iconic  state  changed  and  there's  an  unexpired  timeout
       pending, cancel the timeout and invoke ringgg() to register the
       correct timeout for the new visibility state. */

    if (liconic != iconic) {
	ringgg(toplevel, NULL);
    }
}

/*  TESTON  --	Activate test mode.  */

static void teston(w, client_data, call_data)
  Widget w;
  XtPointer client_data, call_data;
{
    testmode = TRUE;
    faketime = 0.0;
}

/*  TESTOFF  --  Terminate test mode.  */

static void testoff(w, client_data, call_data)
  Widget w;
  XtPointer client_data, call_data;
{
    testmode = FALSE;
    nptime = 0.0;		      /* Force recomputation of phase table */
    expose(NULL, NULL, NULL);	      /* Force complete redisplay */
}

/*  MAIN  --  Main program.  */

int main(argc, argv)
  int argc;
  char *argv[];
{
    int c;
    extern int opterr;
    Widget con, con1, con2, con3, con4, con5, con6, con7, testbutton;
    char *igerr;
    Arg wargs[10];

    OlToolkitInitialize((XtPointer) NULL);
    toplevel = XtAppInitialize(&app, "XMoontool",
			       (XrmOptionDescList) NULL,
			       0, &argc, argv, (String *) NULL,
			       (ArgList) NULL, 0);
#ifdef OLDWAY
    p = XCreateBitmapFromData(XtDisplay(toplevel),
			      RootWindowOfScreen(XtScreen(toplevel)),
			      moon_icon_bits, moon_icon_width,
			      moon_icon_height);
    p1 = XCreateBitmapFromData(XtDisplay(toplevel),
			      RootWindowOfScreen(XtScreen(toplevel)),
			      moon_icon_bits, moon_icon_width,
			      moon_icon_height);
#else
    p = XCreatePixmap(XtDisplay(toplevel),
		      RootWindowOfScreen(XtScreen(toplevel)),
		      moon_icon_width,
		      moon_icon_height,
		      1);
    p1 = XCreatePixmap(XtDisplay(toplevel),
		       RootWindowOfScreen(XtScreen(toplevel)),
		       moon_icon_width,
		       moon_icon_height,
		       1);
#endif
    XtVaSetValues(toplevel,
	XtNtitle, MOON_TITLE,
	XtNiconPixmap, (XtArgVal) p,
	NULL);

    igerr = icongeom(toplevel, &argc, argv);
    if (igerr != NULL) {
        fprintf(stderr, "Error: %s\n", igerr);
	return 2;
    }

    XtAddEventHandler(toplevel, ExposureMask | StructureNotifyMask,
		      TRUE, eproc, NULL);
    con1 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				 toplevel, NULL, 0);
    XtVaSetValues(con1,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNvSpace, (XtArgVal) 0,
	NULL);

    con3 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				 con1, NULL, 0);
    XtVaSetValues(con3,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNcenter, (XtArgVal) TRUE,
	XtNalignCaptions, (XtArgVal) FALSE,
	XtNvSpace, (XtArgVal) 0,
	XtNmeasure, (XtArgVal) 2,
	NULL);

    con2 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				 con3, NULL, 0);
    XtVaSetValues(con2,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNcenter, (XtArgVal) FALSE,
	XtNalignCaptions, (XtArgVal) FALSE,
	XtNsameSize, (XtArgVal) OL_ROWS,
	XtNvSpace, (XtArgVal) 0,
	XtNmeasure, (XtArgVal) 2,
	NULL);

    con4 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				 con3, NULL, 0);
    XtVaSetValues(con4,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNvSpace, (XtArgVal) 0,
	NULL);

    con7 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				 con1, NULL, 0);
    XtVaSetValues(con7,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNcenter, (XtArgVal) FALSE,
	XtNalignCaptions, (XtArgVal) FALSE,
	XtNsameSize, (XtArgVal) OL_ROWS,
	XtNvSpace, (XtArgVal) 0,
	XtNmeasure, (XtArgVal) 2,
	NULL);

    con5 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				 con1, NULL, 0);
    XtVaSetValues(con5,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNcenter, (XtArgVal) FALSE,
	XtNalignCaptions, (XtArgVal) FALSE,
	XtNsameSize, (XtArgVal) OL_ROWS,
	XtNvSpace, (XtArgVal) 0,
	XtNmeasure, (XtArgVal) 2,
	NULL);

    canvas = XtCreateManagedWidget("canvas", stubWidgetClass, con4,
	NULL, 0);

    XtVaSetValues(canvas,
	XtNheight, 64,
	XtNwidth, 64,
	XtNexpose, expose,
	XtNresize, resize,
	NULL);

    XtSetArg(wargs[0], XtNforeground, &gcv.foreground);
    XtSetArg(wargs[1], XtNbackground, &gcv.background);

    XtGetValues(canvas, wargs, 2);

    gcv.function = GXcopy;
    gc = XtGetGC(toplevel, GCForeground | GCBackground | GCFunction, &gcv);

    /* Graphics context for writing into icon pixmaps. */
    gc1 = XCreateGC(XtDisplay(toplevel), p,
		    GCForeground | GCBackground | GCFunction, &gcv);

    testbutton = XtCreateManagedWidget("Test", rectButtonWidgetClass,
	con4, NULL, 0);

    XtVaSetValues(testbutton,
	XtNlabelJustify, OL_CENTER,
	NULL);
    XtAddCallback(testbutton, XtNselect, teston, NULL);
    XtAddCallback(testbutton, XtNunselect, testoff, NULL);

    con = XtCreateManagedWidget("control", controlAreaWidgetClass,
				con5, NULL, 0);
    XtVaSetValues(con,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNcenter, (XtArgVal) FALSE,
	XtNalignCaptions, (XtArgVal) FALSE,
	XtNsameSize, (XtArgVal) OL_ROWS,
	XtNvSpace, (XtArgVal) 0,
	XtNmeasure, (XtArgVal) 2,
	NULL);

    con6 = XtCreateManagedWidget("control", controlAreaWidgetClass,
				con5, NULL, 0);
    XtVaSetValues(con6,
	XtNlayoutType, (XtArgVal) OL_FIXEDCOLS,
	XtNcenter, (XtArgVal) FALSE,
	XtNalignCaptions, (XtArgVal) FALSE,
	XtNsameSize, (XtArgVal) OL_ROWS,
	XtNvSpace, (XtArgVal) 0,
	XtNmeasure, (XtArgVal) 2,
	NULL);

    memset(olabel, 0, sizeof olabel);
    for (c = 0; c < Nlabels; c++) {
	Widget lpanel, lentry;

        lpanel = XtCreateManagedWidget("llabel", staticTextWidgetClass,
	    c < 6 ? con2 : (c < 12 ? con7 : con), NULL, 0);
	XtVaSetValues(lpanel,
	    XtNstring, (XtArgVal) labels[c],
	    XtNalignment, (XtArgVal) OL_RIGHT,
	    NULL);
        wlabel[c] = XtCreateManagedWidget("lvalue", staticTextWidgetClass,
	    c < 6 ? con2: (c < 12 ? con7 : con), NULL, 0);
    }
    for (c = 0; c < 5; c++) {
	Widget lpanel, lentry;

        lpanel = XtCreateManagedWidget("llabel", staticTextWidgetClass,
	    con6, NULL, 0);
	XtVaSetValues(lpanel,
            XtNstring, (XtArgVal) (c == 0 || c == 4) ? "Lunation:" : "",
	    NULL);
	loonie[c > 0 ? 1 : 0] =
            XtCreateManagedWidget("lvalue", staticTextWidgetClass,
		con6, NULL, 0);
    }

    opterr = 0;
    while ((c = getopt(argc, argv, "cmt")) != EOF) {
	switch (c) {
            case 't':                 /* Jump into test mode */
		testmode = TRUE;
		break;

            case 'c':                 /* Force to color mode */
		color_mode = TRUE;
		break;

            case 'm':                 /* Force mono mode */
		color_mode = FALSE;
		break;
	}
    }
    if (opterr) {
	return 2;
    }
    XtVaSetValues(testbutton,
	XtNset, testmode,
	NULL);

    iconic = FALSE;
    ringgg(toplevel, NULL);
    iconic = TRUE;
    XtRealizeWidget(toplevel);
    XtAppMainLoop(app);

    return 0;
}

/*  ROP  --  Perform raster op on icon.  */

static void rop(sx, sy, lx)
  int sx, sy, lx;
{
    int rowoff = sy * (moon_icon_width / 8), i;

    for (i = sx; i < (sx + lx); i++) {
	moon_icon_work[rowoff + (i / 8)] =
	    (moon_icon_work[rowoff + (i / 8)] & ~(1 << (i & 7))) |
	    (moon_icon_bits[rowoff + (i / 8)] &
		(1 << (i & 7)));
    }
}

/*   DRAWMOON  --  Construct icon for moon, given phase of moon.  */

static void drawmoon(ph, aom_d, aom_h, aom_m)
  double ph;
  int aom_d, aom_h, aom_m;
{
    int i, lx, rx;
    double cp, xscale;
    static char tbuf[20];
    Pixmap np;

#define RADIUS		27.0
#define IRADIUS 	27
#define OFFSET		28
#define CENTER		32

    if (!XtIsRealized(toplevel)) {
	return;
    }

    if (color_mode == FALSE) {
	memset(moon_icon_work, 0xFF, sizeof moon_icon_work);
    } else {
#ifdef ZZZ
		if (dark_pr == (Pixrect *)0) {
			Pixrect *stencil_pr;
			int	x, y;

			dark_pr = mem_create(64, 64, 8);
			stencil_pr = mem_create(64, 64, 1);

			for (y = 0; y < 64; y++) {
				for (x = 0; x < 64; x++) {
					if (pr_get(src_pr, x, y) == 0) {
						pr_put(stencil_pr, x, y, 0);
					} else {
						pr_put(stencil_pr, x, y, 1);
					}
				}
			}

			/* Clear the pixrect */
			pr_rop(dark_pr, 0, 0, 64, 64, PIX_CLR,
				(Pixrect *)0, 0, 0);

			/* turn on bits in upper plane that match stencil */
			pr_stencil(dark_pr, 0, 0, 64, 64,
				PIX_SRC | PIX_COLOR(4),
				stencil_pr, 0, 0, (Pixrect *)0, 0, 0);

                        /* OR in moon's image in lower planes */
			pr_stencil(dark_pr, 0, 0, 64, 64, PIX_SRC | PIX_DST,
				stencil_pr, 0, 0, src_pr, 0, 0);

			pr_destroy(stencil_pr);        /* no longer needed */

			/* The Apollo 11 Commemorative Red Dot */
			pr_put(dark_pr, 41, 29, 4);
		}

		/* slap in the image of the moon in darkness */
		pr_rop(dst_pr, 0, 0, 64, 64, PIX_SRC, dark_pr, 0, 0);
#endif
    }

    /* Allow the moon to be completely dark for a few hours when new. */

    if ((color_mode == TRUE) && ((ph < 0.01) || (ph > 0.99))) {
	    return;
    }

    xscale = cos(2 * PI * ph);
    for (i = 0; i < IRADIUS; i++) {
	cp = RADIUS * cos(asin((double) i / RADIUS));
	if (ph < 0.5) {
	    rx = CENTER + cp;
	    lx = CENTER + xscale * cp;
	} else {
	    lx = CENTER - cp;
	    rx = CENTER - xscale * cp;
	}

	/* We  now know the left and right  endpoints of the scan line
	   for this y  coordinate.   We  raster-op  the  corresponding
	   scanlines  from  the  source  pixrect  to  the  destination
	   pixrect, offsetting to properly place it in the pixrect and
	   reflecting vertically. */

	rop(lx, OFFSET + i, (rx - lx) + 1);
	if (i != 0) {
	    rop(lx, OFFSET - i, (rx - lx) + 1);
	}

    }
    if (aom_d == 0) {
        sprintf(tbuf, "%dh %dm", aom_h, aom_m);
    } else {
        sprintf(tbuf, "%dd %dh", aom_d, aom_h);
    }

    /* The  following  silly  little dance where  we double buffer the
       icon in two pixmaps and swap them back and forth has nothing to
       do  with  efficiency.   Rather, it's a work-around for the fact
       that X doesn't update the icon unless you pass it it  different
       address	for  the icon pixmap than the one it had before.  Note
       also that  we  don't  update  the  icon  unless  it's  actually
       changed.    This  not  only  saves  time,  it  also  eliminates
       unnecessary blinking of the icon on the display due to nugatory
       refreshes. */

    if (memcmp(moon_icon_work, moon_icon_last, sizeof moon_icon_last) != 0) {
	memcpy(moon_icon_last, moon_icon_work, sizeof moon_icon_last);
	np = XCreateBitmapFromData(XtDisplay(toplevel),
				   RootWindowOfScreen(XtScreen(toplevel)),
				   moon_icon_work, moon_icon_width,
				   moon_icon_height);
	XCopyPlane(XtDisplay(toplevel), np, p1, gc1, 0, 0,
	    moon_icon_width, moon_icon_height, 0, 0, 1);
	if (XtIsRealized(canvas)) {
	    XCopyPlane(XtDisplay(canvas), np, XtWindow(canvas), gc, 0, 0,
		      moon_icon_width, moon_icon_height, 0, 0, 1);
	}
	XFreePixmap(XtDisplay(toplevel), np);
	XtVaSetValues(toplevel,
	    XtNiconPixmap, (XtArgVal) p1,
	    NULL);

	np = p;
	p = p1;
	p1 = np;
    }

    /* Update the date of the moon in the icon label if it's changed. */

    if (strcmp(tbuf, icon_label_last) != 0) {
	strcpy(icon_label_last, tbuf);
	XtVaSetValues(toplevel,
	    XtNiconName, tbuf,
	    NULL);
    }
}

/*  RINGGG  --	Update	status	on  interval  timer  ticks  and redraw
		window if needed.  */

#define prt(y) if (strcmp(olabel[y - 1], tbuf) != 0) {                \
       XtVaSetValues(wlabel[y - 1], XtNstring, (XtArgVal) tbuf, NULL); \
       strcpy(olabel[y - 1], tbuf); }

#define prl(y) if (strcmp(luabel[y - 1], tbuf) != 0) {                \
       XtVaSetValues(loonie[y - 1], XtNstring, (XtArgVal) tbuf, NULL); \
       strcpy(luabel[y - 1], tbuf); }

#define EPL(x) (x), (x) == 1 ? "" : "s"
#define APOS(x) (x + 13)

static void ringgg(client_data, id)
  XtPointer client_data;
  XtIntervalId *id;
{
    int lunation;
    int i, yy, mm, dd, hh, mmm, ss;
    int aom_d, aom_h, aom_m;
    int op;
    long t;
    double jd, p, aom, cphase, cdist, cangdia, csund, csuang;
    double phasar[5];
    char tbuf[80];
    struct tm *gm;

    time(&t);
    jd = jtime((gm = gmtime(&t)));
    if (testmode) {
	if (faketime == 0.0) {
	    faketime = jd + 1;
	} else {
	    faketime += 1.0 / 24;
	}
	jd = faketime;
    }

    p = phase(jd, &cphase, &aom, &cdist, &cangdia, &csund, &csuang);
    aom_d = (int) aom;
    aom_h = (int) (24 * (aom - floor(aom)));
    aom_m = (int) (1440 * (aom - floor(aom))) % 60;

    drawmoon(p, aom_d, aom_h, aom_m);

    /* If we were called preemptively, cancel the pending unexpired
       timeout. */

    if (id == NULL && tid != NULL) {
	XtRemoveTimeOut(tid);
    }

    tid = XtAppAddTimeOut(app, testmode ? 150 :
	(id == NULL ? 10 : ((iconic ? CLOSED_SECS : OPEN_SECS) * 1000)),
	ringgg, (Widget) client_data);

    if (iconic) {

	/* Iconic */

	    return;
    }

    /* Update textual information for open window */

    sprintf(tbuf, "%.5f", jd + 0.5);  /* Julian date */
    prt(1);

    if (testmode) {		      /* Universal time */
	jyear(jd, &yy, &mm, &dd);
	jhms(jd, &hh, &mmm, &ss);
        sprintf(tbuf, "%2d:%02d:%02d %2d %s %d",
		hh, mmm, ss, dd, moname [mm - 1], yy);
    } else {
        sprintf(tbuf, "%2d:%02d:%02d %2d %s %d",
		gm->tm_hour, gm->tm_min, gm->tm_sec,
		gm->tm_mday, moname [gm->tm_mon], gm->tm_year + 1900);
    }
    prt(2);

    if (testmode == FALSE) {	      /* Ignore local time in test mode */
	gm = localtime(&t);

	/* Local time */

        sprintf(tbuf, "%2d:%02d:%02d %2d %s %d",
		gm->tm_hour, gm->tm_min, gm->tm_sec,
		gm->tm_mday, moname [gm->tm_mon], gm->tm_year + 1900);
	prt(3);
    }

    /* Information about the Moon */

    /* Age of moon */

    sprintf(tbuf, "%d day%s, %d hour%s, %d minute%s.",
	    EPL(aom_d), EPL(aom_h), EPL(aom_m));
    prt(5);

    /* Moon phase */

    sprintf(tbuf, "%d%%   (0%% = New, 100%% = Full)",
	    (int) (cphase * 100));
    while (strlen(tbuf) < 40) {
        strcat(tbuf, " ");
    }
    prt(6);

    /* Moon distance */

    sprintf(tbuf, "%ld kilometres, %.1f Earth radii.",
	    (long) cdist, cdist / earthrad);
    prt(7);

    /* Moon subtends */

    sprintf(tbuf, "%.4f degrees.", cangdia);
    prt(8);

    /* Information about the Sun */

    /* Sun's distance */

    sprintf(tbuf, "%.0f kilometres, %.3f astronomical units.",
	    csund, csund / sunsmax);
    prt(10);

    /* Sun subtends */

    sprintf(tbuf, "%.4f degrees.", csuang);
    prt(11);

    /* Calculate times of phases of this lunation.  This is
       sufficiently time-consuming that we only do it once a month. */

    if (jd > nptime) {
	phasehunt(jd + 0.5, phasar);
	lunation = floor(((phasar[0] + 7) - lunatbase) / synmonth) + 1;

	for (i = 0; i < 5; i++) {
	    fmt_phase_time(phasar[i], tbuf);
	    prt(APOS(i));
	}
	nptime = phasar[4];

	/* Edit lunation numbers into cells reserved for them. */

        sprintf(tbuf, "%d", lunation);
	prl(1);
        sprintf(tbuf, "%d", lunation + 1);
	prl(2);
    }
    return;
}
#undef APOS


/*  FMT_PHASE_TIME  --	Format	the  provided  julian  date  into  the
			provided  buffer  in  UTC  format  for	screen
			display  */

static void fmt_phase_time(utime, buf)
    double  utime;
    char    *buf;
{
    int yy, mm, dd, hh, mmm, ss;

    jyear(utime, &yy, &mm, &dd);
    jhms(utime, &hh, &mmm, &ss);
    sprintf(buf, "%2d:%02d UTC %2d %s %d",
	hh, mmm, dd, moname [mm - 1], yy);
}


/*  JDATE  --  Convert internal GMT date and time to  Julian  day  and
	       fraction.  */

static long jdate(t)
  struct tm *t;
{
    long c, m, y;

    y = t->tm_year + 1900;
    m = t->tm_mon + 1;
    if (m > 2) {
	m = m - 3;
    } else {
	m = m + 9;
	y--;
    }
    c = y / 100L;		      /* Compute century */
    y -= 100L * c;
    return (t->tm_mday + (c * 146097L) / 4 + (y * 1461L) / 4 +
	   (m * 153L + 2) / 5 + 1721119L);
}


/*  JTIME  --  Convert internal GMT  date  and	time  to  astronomical
	       Julian	time  (i.e. Julian  date  plus	day  fraction,
	       expressed as a double).	*/

static double jtime(t)
  struct tm *t;
{
    return (jdate(t) - 0.5) + 
	   (t->tm_sec + 60 * (t->tm_min + 60 * t->tm_hour)) / 86400.0;
}


/*  JYEAR  --  Convert	Julian	date  to  year,  month, day, which are
	       returned via integer pointers to integers.  */

static void jyear(td, yy, mm, dd)
  double  td;
  int *yy, *mm, *dd;
{
    double j, d, y, m;

    td += 0.5;			      /* Astronomical to civil */
    j = floor(td);
    j = j - 1721119.0;
    y = floor(((4 * j) - 1) / 146097.0);
    j = (j * 4.0) - (1.0 + (146097.0 * y));
    d = floor(j / 4.0);
    j = floor(((4.0 * d) + 3.0) / 1461.0);
    d = ((4.0 * d) + 3.0) - (1461.0 * j);
    d = floor((d + 4.0) / 4.0);
    m = floor(((5.0 * d) - 3) / 153.0);
    d = (5.0 * d) - (3.0 + (153.0 * m));
    d = floor((d + 5.0) / 5.0);
    y = (100.0 * y) + j;
    if (m < 10.0) {
	m = m + 3;
    } else {
	m = m - 9;
	y = y + 1;
    }
    *yy = y;
    *mm = m;
    *dd = d;
}


/*  JHMS  --  Convert Julian time to hour, minutes, and seconds.  */

static void jhms(j, h, m, s)
  double j;
  int *h, *m, *s;
{
    long ij;

    j += 0.5;			      /* Astronomical to civil */
    ij = (j - floor(j)) * 86400.0;
    *h = ij / 3600L;
    *m = (ij / 60L) % 60L;
    *s = ij % 60L;
}


/*  MEANPHASE  --  Calculates  time  of  the mean new Moon for a given
		   base date.  This argument K to this function is the
		   precomputed synodic month index, given by:
  
			  K = (year - 1900) * 12.3685
  
		   where year is expressed as a year and fractional year.  */

static double meanphase(sdate, k)
  double sdate, k;
{
    double t, t2, t3, nt1;

    /* Time in Julian centuries from 1900 January 0.5 */
    t = (sdate - 2415020.0) / 36525;
    t2 = t * t; 		      /* Square for frequent use */
    t3 = t2 * t;		      /* Cube for frequent use */

    nt1 = 2415020.75933 + synmonth * k
	    + 0.0001178 * t2
	    - 0.000000155 * t3
	    + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

    return nt1;
}


/*  TRUEPHASE  --  Given a K value used to determine the mean phase of
		   the new moon, and a phase selector (0.0, 0.25, 0.5,
		   0.75), obtain the true, corrected phase time.  */

static double truephase(k, phase)
  double k, phase;
{
    double t, t2, t3, pt, m, mprime, f;
    int apcor = FALSE;

    k += phase; 		      /* Add phase to new moon time */
    t = k / 1236.85;		      /* Time in Julian centuries from
					 1900 January 0.5 */
    t2 = t * t; 		      /* Square for frequent use */
    t3 = t2 * t;		      /* Cube for frequent use */
    pt = 2415020.75933		      /* Mean time of phase */
	 + synmonth * k
	 + 0.0001178 * t2
	 - 0.000000155 * t3
	 + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

    m = 359.2242                      /* Sun's mean anomaly */
	+ 29.10535608 * k
	- 0.0000333 * t2
	- 0.00000347 * t3;
    mprime = 306.0253                 /* Moon's mean anomaly */
	+ 385.81691806 * k
	+ 0.0107306 * t2
	+ 0.00001236 * t3;
    f = 21.2964                       /* Moon's argument of latitude */
	+ 390.67050646 * k
	- 0.0016528 * t2
	- 0.00000239 * t3;
    if ((phase < 0.01) || (abs(phase - 0.5) < 0.01)) {

       /* Corrections for New and Full Moon */

       pt +=	 (0.1734 - 0.000393 * t) * dsin(m)
		+ 0.0021 * dsin(2 * m)
		- 0.4068 * dsin(mprime)
		+ 0.0161 * dsin(2 * mprime)
		- 0.0004 * dsin(3 * mprime)
		+ 0.0104 * dsin(2 * f)
		- 0.0051 * dsin(m + mprime)
		- 0.0074 * dsin(m - mprime)
		+ 0.0004 * dsin(2 * f + m)
		- 0.0004 * dsin(2 * f - m)
		- 0.0006 * dsin(2 * f + mprime)
		+ 0.0010 * dsin(2 * f - mprime)
		+ 0.0005 * dsin(m + 2 * mprime);
       apcor = TRUE;
    } else if ((abs(phase - 0.25) < 0.01 || (abs(phase - 0.75) < 0.01))) {
       pt +=	 (0.1721 - 0.0004 * t) * dsin(m)
		+ 0.0021 * dsin(2 * m)
		- 0.6280 * dsin(mprime)
		+ 0.0089 * dsin(2 * mprime)
		- 0.0004 * dsin(3 * mprime)
		+ 0.0079 * dsin(2 * f)
		- 0.0119 * dsin(m + mprime)
		- 0.0047 * dsin(m - mprime)
		+ 0.0003 * dsin(2 * f + m)
		- 0.0004 * dsin(2 * f - m)
		- 0.0006 * dsin(2 * f + mprime)
		+ 0.0021 * dsin(2 * f - mprime)
		+ 0.0003 * dsin(m + 2 * mprime)
		+ 0.0004 * dsin(m - 2 * mprime)
		- 0.0003 * dsin(2 * m + mprime);
       if (phase < 0.5)
	  /* First quarter correction */
	  pt += 0.0028 - 0.0004 * dcos(m) + 0.0003 * dcos(mprime);
       else
	  /* Last quarter correction */
	  pt += -0.0028 + 0.0004 * dcos(m) - 0.0003 * dcos(mprime);
       apcor = TRUE;
    }
    if (!apcor) {
	fprintf(stderr,
            "TRUEPHASE called with invalid phase selector.\n");
	abort();
    }
    return pt;
}


/*   PHASEHUNT	--  Find time of phases of the moon which surround the
		    current date.  Five phases are found, starting and
		    ending with the new moons which bound the  current
		    lunation.  */

static void phasehunt(sdate, phases)
  double sdate;
  double phases[5];
{
    double adate, k1, k2, nt1, nt2;
    int yy, mm, dd;

    adate = sdate - 45;

    jyear(adate, &yy, &mm, &dd);
    k1 = floor((yy + ((mm - 1) * (1.0 / 12.0)) - 1900) * 12.3685);

    adate = nt1 = meanphase(adate, k1);
    while (TRUE) {
	adate += synmonth;
	k2 = k1 + 1;
	nt2 = meanphase(adate, k2);
	if (nt1 <= sdate && nt2 > sdate)
	    break;
	nt1 = nt2;
	k1 = k2;
    }
    phases[0] = truephase(k1, 0.0);
    phases[1] = truephase(k1, 0.25);
    phases[2] = truephase(k1, 0.5);
    phases[3] = truephase(k1, 0.75);
    phases[4] = truephase(k2, 0.0);
}


/*  KEPLER  --	 Solve the equation of Kepler.	*/

static double kepler(m, ecc)
  double m, ecc;
{
    double e, delta;
#define EPSILON 1E-6

    e = m = torad(m);
    do {
	delta = e - ecc * sin(e) - m;
	e -= delta / (1 - ecc * cos(e));
    } while (abs(delta) > EPSILON);
    return e;
}

/*  PHASE  --  Calculate phase of moon as a fraction:
  
    The  argument  is  the  time  for  which  the  phase is requested,
    expressed as a Julian date and fraction.  Returns  the  terminator
    phase  angle  as a percentage of a full circle (i.e., 0 to 1), and
    stores into pointer arguments  the	illuminated  fraction  of  the
    Moon's  disc, the Moon's age in days and fraction, the distance of
    the Moon from the centre of the Earth, and	the  angular  diameter
    subtended  by the Moon as seen by an observer at the centre of the
    Earth.
*/

static double phase(pdate, pphase, mage, dist, angdia, sudist, suangdia)
  double  pdate;
  double  *pphase;		      /* Illuminated fraction */
  double  *mage;		      /* Age of moon in days */
  double  *dist;		      /* Distance in kilometres */
  double  *angdia;		      /* Angular diameter in degrees */
  double  *sudist;		      /* Distance to Sun */
  double  *suangdia;                  /* Sun's angular diameter */
{

    double Day, N, M, Ec, Lambdasun, ml, MM, MN, Ev, Ae, A3, MmP,
	   mEc, A4, lP, V, lPP, NP, y, x, Lambdamoon, BetaM,
	   MoonAge, MoonPhase,
	   MoonDist, MoonDFrac, MoonAng, MoonPar,
	   F, SunDist, SunAng;

    /* Calculation of the Sun's position */

    Day = pdate - epoch;		    /* Date within epoch */
    N = fixangle((360 / 365.2422) * Day);   /* Mean anomaly of the Sun */
    M = fixangle(N + elonge - elongp);	    /* Convert from perigee
					    co-ordinates to epoch 1980.0 */
    Ec = kepler(M, eccent);		    /* Solve equation of Kepler */
    Ec = sqrt((1 + eccent) / (1 - eccent)) * tan(Ec / 2);
    Ec = 2 * todeg(atan(Ec));		    /* True anomaly */
    Lambdasun = fixangle(Ec + elongp);      /* Sun's geocentric ecliptic
						    longitude */
    /* Orbital distance factor */
    F = ((1 + eccent * cos(torad(Ec))) / (1 - eccent * eccent));
    SunDist = sunsmax / F;		    /* Distance to Sun in km */
    SunAng = F * sunangsiz;                 /* Sun's angular size in degrees */


    /* Calculation of the Moon's position */

    /* Moon's mean longitude */
    ml = fixangle(13.1763966 * Day + mmlong);

    /* Moon's mean anomaly */
    MM = fixangle(ml - 0.1114041 * Day - mmlongp);

    /* Moon's ascending node mean longitude */
    MN = fixangle(mlnode - 0.0529539 * Day);

    /* Evection */
    Ev = 1.2739 * sin(torad(2 * (ml - Lambdasun) - MM));

    /* Annual equation */
    Ae = 0.1858 * sin(torad(M));

    /* Correction term */
    A3 = 0.37 * sin(torad(M));

    /* Corrected anomaly */
    MmP = MM + Ev - Ae - A3;

    /* Correction for the equation of the centre */
    mEc = 6.2886 * sin(torad(MmP));

    /* Another correction term */
    A4 = 0.214 * sin(torad(2 * MmP));

    /* Corrected longitude */
    lP = ml + Ev + mEc - Ae + A4;

    /* Variation */
    V = 0.6583 * sin(torad(2 * (lP - Lambdasun)));

    /* True longitude */
    lPP = lP + V;

    /* Corrected longitude of the node */
    NP = MN - 0.16 * sin(torad(M));

    /* Y inclination coordinate */
    y = sin(torad(lPP - NP)) * cos(torad(minc));

    /* X inclination coordinate */
    x = cos(torad(lPP - NP));

    /* Ecliptic longitude */
    Lambdamoon = todeg(atan2(y, x));
    Lambdamoon += NP;

    /* Ecliptic latitude */
    BetaM = todeg(asin(sin(torad(lPP - NP)) * sin(torad(minc))));

    /* Calculation of the phase of the Moon */

    /* Age of the Moon in degrees */
    MoonAge = lPP - Lambdasun;

    /* Phase of the Moon */
    MoonPhase = (1 - cos(torad(MoonAge))) / 2;

    /* Calculate distance of moon from the centre of the Earth */

    MoonDist = (msmax * (1 - mecc * mecc)) /
       (1 + mecc * cos(torad(MmP + mEc)));

    /* Calculate Moon's angular diameter */

    MoonDFrac = MoonDist / msmax;
    MoonAng = mangsiz / MoonDFrac;

    /* Calculate Moon's parallax */

    MoonPar = mparallax / MoonDFrac;

    *pphase = MoonPhase;
    *mage = synmonth * (fixangle(MoonAge) / 360.0);
    *dist = MoonDist;
    *angdia = MoonAng;
    *sudist = SunDist;
    *suangdia = SunAng;
    return fixangle(MoonAge) / 360.0;
}
