/*

    A Moon for the Sun

    Release 2.5

    Designed and implemented by John Walker in December 1987,
    Revised and updated in February of 1988.
    Revised and updated again in June of 1988 by Ron Hitchens.
    Revised and updated yet again in July/August of 1989 by Ron Hitchens.

    Make with:

    cc -O moontool.c -o moontool -lm -lsuntool -lsunwindow -lpixrect

    Adding  appropriate  floating  point  options  to your hardware.  This
    program is a SunView tool which displays, as the  icon  for  a  closed
    window,  the  current phase of the Moon.  A subtitle in the icon gives
    the age of the Moon in days  and  hours.   If  called  with  the  "-t"
    switch,  it  rapidly  increments  forward  through time to display the
    cycle of phases.

    If you open the window, additional information is displayed  regarding
    the  Moon.	 The  information  is  generally  accurate  to	within ten
    minutes.

    The algorithms used in this program to calculate the positions Sun and
    Moon as seen from the Earth are given in the book "Practical Astronomy
    With  Your  Calculator"  by  Peter  Duffett-Smith,   Second   Edition,
    Cambridge University Press, 1981.  Ignore the word "Calculator" in the
    title;  this  is  an  essential  reference  if  you're  interested  in
    developing	software  which  calculates  planetary	positions, orbits,
    eclipses, and  the  like.   If  you're  interested  in  pursuing  such
    programming, you should also obtain:

    "Astronomical  Formulae for Calculators" by Jean Meeus, Third Edition,
    Willmann-Bell, 1985.  A must-have.

    "Planetary  Programs  and  Tables  from  -4000  to  +2800"  by  Pierre
    Bretagnon  and Jean-Louis Simon, Willmann-Bell, 1986.  If you want the
    utmost  (outside  of  JPL)  accuracy  for  the  planets,  it's   here.

    "Celestial BASIC" by Eric Burgess, Revised Edition, Sybex, 1985.  Very
    cookbook oriented, and many of the algorithms are hard to dig  out	of
    the turgid BASIC code, but you'll probably want it anyway.

    Many of these references can be obtained from Willmann-Bell, P.O.  Box
    35025,  Richmond,  VA 23235, USA.  Phone: (804) 320-7016.  In addition
    to their own publications, they stock most of the standard	references
    for mathematical and positional astronomy.

    This program was written by:

	John Walker
	http://www.fourmilab.ch/

    This  program is in the public domain: "Do what thou wilt shall be the
    whole of the law".  I'd appreciate  receiving  any  bug  fixes  and/or
    enhancements,  which  I'll  incorporate  in  future  versions  of  the
    program.  Please leave the original attribution information intact	so
    that credit and blame may be properly apportioned.

----------------

	History:
	--------
	June 1988	Version 2.0 posted to usenet by John Walker

	June 1988	Modified by Ron Hitchens to produce version 2.1
			modified icon generation to show surface texture
                         on visible moon face.  Eliminated "illegal" direct
			 modification of icon image memory.
			added a menu to allow switching in and out of
			 test mode, for entertainment value mostly.
                        reworked timer behaviour so that process doesn't
			 wake up unnecessarily.
			trap sigwinch signals to notice more easily when the
			 tool opens and closes.
			modified layout of information in open window display
			 to reduce the amount of pixels modified in each
			 update.  Batched pixwin updates so that only one
			 screen rasterop per cycle is done.
			changed open window to display white-on-black for a
			 more aesthetic look, and to suggest the effect of
			 looking at the moon in the nighttime sky.
			setup default tool and canvas colors to be the same
			 as B&W monochrome, for those us lucky enough to have
			 color monitors and who have the default monochrome
			 colors set to something other than B&W (I like green
			 on black myself)
			various code reformatting and pretty-printing to suit
			 my own coding taste (I got a bit carried away).
			code tweaking to make lint happy.
			returned my hacked version to John.
			(but he never got it)

	July 1989	Modified further for color displays.  On a color Sun,
			 four colors will be used for the canvas and the icon.
			 Rather than just show the illuminated portion of
			 the moon, a color icon will also show the darkened
			 portion in a dark blue shade.	The text on the icon
                         will also be drawn in a nice "buff" color, since there
			 was one more color left to use.
                        Add two command line args, "-c" and "-m" to explicitly
			 specify color or monochrome mode.  If neither are
                         given, moontool will try to determine if it's
			 running on a color or mono display by looking at the
                         depth of the frame's pixwin's pixrect.  This is not
			 always reliable on a Prism-type framebuffer like my
			 3/60C, so these two args will force one or the
			 other mode.
			Use getopt to parse the args.
			Change the tool menu slightly to use only one item
			 for switching in and out of test mode.
			A little more lint picking.

	July 1989	Modified a little bit more a few days later to use 8
			 colors and an accurate grey-scale moon face created
			 by Joe Hitchens on an Amiga.
			Added the -b option to draw a one pixel border around
			 the icon.  I like it, but it may not mesh well with
			 some backgrounds or adjacent icons.
			Added The Apollo 11 Commemorative Red Dot, to show
			 where Neil and Buzz went on vacation a few years ago.
			Updated man page.

        August 1989     Received version 2.3 of John Walker's original code.
			 Rolled in bug fixes to astronomical algorithms:

                         2.1  6/16/88   Bug fix.  Table of phases didn't update
					at the moment of the new moon.	Call on
                                        phasehunt didn't convert civil Julian
					date to astronomical Julian date.
					Reported by Dag Bruck
					 (dag@control.lth.se).

			 2.2  N/A	(superseded by my code)

			 2.3  6/7/89	Bug fix.  Table of phases skipped the
					phases for July 1989.  This occurred
					due to sloppy maintenance of the
					synodic month index in the interchange
					of information between phasehunt() and
					meanphase().  I simplified and
					corrected the handling of the month
					index as phasehunt() steps along and
					removed unneeded code from meanphase().
					Reported by Bill Randle of Tektronix.
					 (billr@saab.CNA.TEK.COM).

                        I've taken it upon myself to call this version 2.4

	Ron Hitchens
		ronbo@vixen.uucp
		...!uunet!cs.utah.edu!caeco!vixen!ronbo
		hitchens@cs.utexas.edu

	December 1999	Minor Y2K fix.	Release 2.5.

*/


#include <stdio.h>
#include <math.h>

#include <suntool/sunview.h>
#include <suntool/canvas.h>


#define UNKNOWN 	-2

#define MOON_TITLE \
        " A Moon for the Sun       by John Walker   v2.5"

#define TINYFONT        "/usr/lib/fonts/fixedwidthfonts/screen.r.7"

#define CLOSED_SECS	120		/* update interval when tool closed */
#define CLOSED_USECS	0
#define OPEN_SECS	1		/* update interval when open */
#define OPEN_USECS	0

/*
 * define standard B/W monochrome colors as defaults in case we're running
 * on a color system with the monochrome colors set differently
 */
#define FG_DEFAULT	{ 0, 0, 0 }		/* black */
#define BG_DEFAULT	{ 255, 255, 255 }	/* white */

/* define the colormap for use on a color monitor */

#define MOON_CMS_NAME   "moontoolcms"
#define MOON_CMS_LENGTH 8

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


/*
 * Moon image.  This is a standard icon-sized picture of the moon's face.
 * The visible part, as calculated by the current time, is extracted from
 * this image to create the displayed image.
 */

static short moon_img [] = {
#include "moon.icon"
};
mpr_static(moon_mpr, 64, 64, 1, moon_img);

static short color_moon_img [] = {
#include "colormoon.icon"
};
mpr_static(color_moon_mpr, 64, 64, 8, color_moon_img);


static Pixrect	*virgin_mpr = &moon_mpr;	/* pristine moon image */
static Pixrect	*icon_mpr;			/* actual displayed pixrect */

static Frame    frame;                  /* handle for the tool's frame */
static Icon     moon_icon;              /* handle for the frame's icon info */
static Pixfont	*pfont; 		/* pointer to tiny icon font */
static Canvas	canvas; 		/* handle for the canvas */
static Menu	canvas_menu;		/* handle for the canvas menu */
static Pixwin   *fpw;                   /* pointer to the frame's pixwin */
static Pixwin   *cpw;                   /* pointer to the canvas' pixwin */
static int	charhgt, charwid;	/* default std font height/width */
static int	info_col;		/* canvas x-coord to draw lunar info */

static struct singlecolor	fg_default = FG_DEFAULT;
static struct singlecolor	bg_default = BG_DEFAULT;

static unsigned char	moon_red [] = COLOR_R;
static unsigned char	moon_green [] = COLOR_G;
static unsigned char	moon_blue [] = COLOR_B;

static int	testmode = FALSE;   /* Rapid warp through time for debugging */
static int	color_mode = UNKNOWN;	/* indicates color/mono mode */
static int	border = FALSE; 	/* draw a border around the icon */
static struct itimerval kickme = {{CLOSED_SECS, CLOSED_USECS}, {0, 1} };

static double	nptime = 0.0;		/* Next new moon time */
/* moved up here as a global so it can be reset when switching modes */

/*
 * these two variables do nothing except provide unique addresses to be used
 * as client handles for registering with the notifier.
 */
static int	timer_client, sig_client;


static char	*moname [] = {
                        "January", "February", "March", "April", "May",
                        "June", "July", "August", "September",
                        "October", "November", "December"
};

static char	*labels [] = {
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


/*  Forward functions  */

static double		jtime(), phase();
static void		phasehunt(), set_mode();
static Notify_value	ringgg(), catch_winch(), catch_destroy();
static void		drawmoon(), jyear(), jhms();

extern long		time();
extern int		fprintf();
extern char		*sprintf();




main (argc, argv)
	int argc;
	char **argv;
{
	int		c;
	struct pixfont	*font;
	extern int	opterr;

	opterr = 0;
        while ((c = getopt (argc, argv, "cmbt")) != EOF) {
		switch (c) {
                case 't':                       /* jump into test mode */
			testmode = TRUE;
			break;

                case 'c':                       /* force to color mode */
			color_mode = TRUE;
			break;

                case 'm':                       /* force mono mode */
			color_mode = FALSE;
			break;

                case 'b':
			border = TRUE;		/* draw border around icon */
			break;
		}
	}

	pfont = pf_open (TINYFONT);

	icon_mpr = mem_create (64, 64, 1);
	pr_rop (icon_mpr, 0, 0, 64, 64, PIX_SRC, &moon_mpr, 0, 0);

	moon_icon = icon_create (ICON_IMAGE, icon_mpr, 0);

	frame = window_create (NULL, FRAME,
		FRAME_LABEL,			MOON_TITLE,
		FRAME_INHERIT_COLORS,		TRUE,
		FRAME_FOREGROUND_COLOR, 	&fg_default,
		FRAME_BACKGROUND_COLOR, 	&bg_default,
		FRAME_ICON,			moon_icon,
		FRAME_ARGS,			argc, argv,
		FRAME_CLOSED,			TRUE,
                WIN_ERROR_MSG,                  "Can't create window.",
		0);
	fpw = (Pixwin *) window_get (frame, WIN_PIXWIN);

	canvas = window_create (frame, CANVAS,
		CANVAS_RETAINED,		FALSE,
		WIN_IGNORE_PICK_EVENT,		MS_RIGHT,
		0);
	cpw = canvas_pixwin (canvas);

	if (color_mode == UNKNOWN) {		/* if not set, try to guess */
		color_mode = check_for_color (fpw);
	}

	if (color_mode == TRUE) {		/* switch to color icon */
		Icon	ic;
		Pixrect *pr;

		set_moon_cms (fpw, cpw);
		virgin_mpr = &color_moon_mpr;
		pr = mem_create (64, 64, 8);
		pr_rop (pr, 0, 0, 64, 64, PIX_SRC, virgin_mpr, 0, 0);
		ic = icon_create (ICON_IMAGE, pr, 0);
		window_set (frame, FRAME_ICON, ic, 0);
		icon_destroy (moon_icon);
		pr_destroy (icon_mpr);
		icon_mpr = pr;
		moon_icon = ic;
	}

	/* must set retained after setting colormap */
	window_set (canvas, CANVAS_RETAINED, TRUE, 0);

	font = (Pixfont *) window_get (canvas, WIN_FONT);
	charwid = font->pf_defaultsize.x;
	charhgt = font->pf_defaultsize.y;
	info_col = charwid * 20;

	window_set (frame,
		WIN_WIDTH,			charwid * 70,
		WIN_HEIGHT,			charhgt * 19,
		0);

	pw_writebackground (cpw, 0, 0,
		(int) window_get (canvas, CANVAS_WIDTH),
		(int) window_get (canvas, CANVAS_HEIGHT),
		(color_mode == TRUE) ? PIX_CLR : PIX_SET);
	paint_labels ();

	canvas_menu = menu_create (
		MENU_ITEM,
                        MENU_STRING,            "Test Mode",
			MENU_VALUE,		TRUE,
			MENU_NOTIFY_PROC,	set_mode,
			0,
                MENU_PULLRIGHT_ITEM,            "Frame",
					(Menu)window_get (frame, WIN_MENU),
		0);
	window_set (frame, WIN_MENU, canvas_menu, 0);

	notify_set_signal_func (&sig_client, catch_winch, SIGWINCH,
		NOTIFY_SYNC);

	notify_interpose_destroy_func (frame, catch_destroy);

	window_main_loop (frame);
}


/*
 * Notification proc for the test/normal menu item.  The value of the menu
 * item is what the test mode flag will be set to.  Each time this func is
 * called, we'll change the menu's name and value so that it will do the
 * opposite thing the next time it is called.
 * We've also explicitly ignored MS_RIGHT pick events for the canvas, which
 * means those events will "fall through" to the frame, which effectively
 * gives the canvas area the same menu as the frame.
 */

/*ARGSUSED*/
static
void
set_mode (m, mi)
	Menu		m;
	Menu_item	mi;
{
	testmode = (int)menu_get (mi, MENU_VALUE);
	if (testmode == FALSE) {
                menu_set (mi, MENU_VALUE, TRUE, MENU_STRING, "Test Mode", 0);
	} else {
                menu_set (mi, MENU_VALUE, FALSE, MENU_STRING, "Normal Mode", 0);
	}

	nptime = 0.0;			/* force lunation info to update */
	/* fake a sigwinch to modify the timer */
	(void)catch_winch ((Notify_client)&sig_client, SIGWINCH, NOTIFY_SYNC);
}


/*
 * Catch window change events.  We'll get at least one winch each time the
 * window opens or closes, as well as when a portion of it is uncovered.
 * We also get a winch when the tool first appears, we depend on this fact to
 * start the timer running.  Each time we get a winch we check the current open
 * state of the tool and set the timer interval appropriately.	The timer
 * we set has a nearly immediate initial trip which will cause a refresh
 * of the icon or open window, then a periodic trip which depends on whether
 * the window is open or closed.
 */

/*ARGSUSED*/
static
Notify_value
catch_winch (client, signal_num, mode)
	Notify_client		client;
	int			signal_num;
	Notify_signal_mode	mode;
{
	if (testmode) {
		kickme = NOTIFY_POLLING_ITIMER; 	/* run flat out */
	} else {
		if ((int) window_get(frame, FRAME_CLOSED) == TRUE) {
			kickme.it_interval.tv_sec = CLOSED_SECS;
			kickme.it_interval.tv_usec = CLOSED_USECS;
		} else {
			kickme.it_interval.tv_sec = OPEN_SECS;
			kickme.it_interval.tv_usec = OPEN_USECS;
		}
		kickme.it_value.tv_sec = 0;	/* immediate initial trip */
		kickme.it_value.tv_usec = 10000;
	}

	/* Set/change the timer to the proper interval for new window state */
	notify_set_itimer_func (&timer_client, ringgg, ITIMER_REAL,
		&kickme, NULL);
	return (NOTIFY_DONE);
}


/*
 *	Interpose on the destroy function so that we can stop the timer before
 *	the windows go away.  This avoids a bunch of irritating ioctl failure
 *	messages being kicked out under certain circumstances.
 */

static
Notify_value
catch_destroy (frame, status)
	Frame		frame;
	Destroy_status	status;
{
	if (status != DESTROY_CHECKING) {
		notify_set_itimer_func (&timer_client, (Notify_value (*)())0,
			ITIMER_REAL, (struct itimerval *)0, NULL);
	}

	return (notify_next_destroy_func (frame, status));
}


/*
 *	Try to determine if we are running on a color system.  This is not
 *	the absolutely definitive method, one should really check plane
 *	group availability bits, but this gets it most of the time.
 */

static
int
check_for_color (pw)
	Pixwin	*pw;
{
	if (pw->pw_pixrect->pr_depth == 1) {
		return (FALSE);
	}

	return (TRUE);
}


/*
 *	Set the color cms for the moontool frame/icon and canvas pixwins
 */

static
set_moon_cms (pw1, pw2)
	Pixwin	*pw1, *pw2;
{
	pw_setcmsname (pw1, MOON_CMS_NAME);
	pw_putcolormap (pw1, 0, MOON_CMS_LENGTH,
		moon_red, moon_green, moon_blue);

	pw_setcmsname (pw2, MOON_CMS_NAME);
	pw_putcolormap (pw2, 0, MOON_CMS_LENGTH,
		moon_red, moon_green, moon_blue);

	return (TRUE);
}


/*
 * DRAWMOON  --  Construct icon for moon, given phase of moon.
 */

static
void
drawmoon (ph, src_pr, dst_pr)
	double		ph;
	Pixrect 	*src_pr, *dst_pr;
{
	register int	i, lx, rx;
	register double cp, xscale;
	static Pixrect	*dark_pr = (Pixrect *)0;
#define RADIUS		27.0
#define IRADIUS 	27
#define OFFSET		28
#define CENTER		32


	if (color_mode == FALSE) {
		/* Clear the destination pixrect to all one-bits (black) */
		pr_rop (dst_pr, 0, 0, 64, 64, PIX_SET, (Pixrect *)0, 0, 0);
	} else {
		if (dark_pr == (Pixrect *)0) {
			Pixrect *stencil_pr;
			int	x, y;

			dark_pr = mem_create (64, 64, 8);
			stencil_pr = mem_create (64, 64, 1);

			for (y = 0; y < 64; y++) {
				for (x = 0; x < 64; x++) {
					if (pr_get (src_pr, x, y) == 0) {
						pr_put (stencil_pr, x, y, 0);
					} else {
						pr_put (stencil_pr, x, y, 1);
					}
				}
			}

			/* Clear the pixrect */
			pr_rop (dark_pr, 0, 0, 64, 64, PIX_CLR,
				(Pixrect *)0, 0, 0);

			/* turn on bits in upper plane that match stencil */
			pr_stencil (dark_pr, 0, 0, 64, 64,
				PIX_SRC | PIX_COLOR (4),
				stencil_pr, 0, 0, (Pixrect *)0, 0, 0);

                        /* OR in moon's image in lower planes */
			pr_stencil (dark_pr, 0, 0, 64, 64, PIX_SRC | PIX_DST,
				stencil_pr, 0, 0, src_pr, 0, 0);

			pr_destroy (stencil_pr);	/* no longer needed */

			/* The Apollo 11 Commemorative Red Dot */
			pr_put (dark_pr, 41, 29, 4);
		}

		/* slap in the image of the moon in darkness */
		pr_rop (dst_pr, 0, 0, 64, 64, PIX_SRC, dark_pr, 0, 0);
	}

	/* Allow the moon to be completely dark for a few hours when new */
	if ((color_mode == TRUE) && ((ph < 0.01) || (ph > 0.99))) {
		return;
	}

	xscale = cos (2 * PI * ph);
	for (i = 0; i < IRADIUS; i++) {
		cp = RADIUS * cos (asin ((double) i / RADIUS));
		if (ph < 0.5) {
			rx = CENTER + cp;
			lx = CENTER + xscale * cp;
		} else {
			lx = CENTER - cp;
			rx = CENTER - xscale * cp;
		}

		/*
		 * We now know the left and right endpoints of the scan line
		 * for this y coordinate.  We raster-op the corresponding
		 * scanlines from the source pixrect to the destination
		 * pixrect, offsetting to properly place it in the pixrect and
		 * reflecting vertically.
		 */
		pr_rop (dst_pr, lx, OFFSET + i, (rx - lx) + 1, 1,
			PIX_SRC, src_pr, lx, OFFSET + i);
		if (i != 0) {
			pr_rop (dst_pr, lx, OFFSET - i, (rx - lx) + 1, 1,
				PIX_SRC, src_pr, lx, OFFSET - i);
		}
	}
}


/*
 * RINGGG  --	Update status on interval timer ticks and redraw
 *		window if needed.
 */

#define prt(y) pw_text(cpw, info_col, charhgt*(y), op, NULL, tbuf)
#define prtxy(x,y) pw_text(cpw, charwid*(y+1), charhgt*(x), op, NULL, tbuf)

#define EPL(x) (x), (x) == 1 ? "" : "s"
#define APOS(x) (x + 13)

/*ARGSUSED*/
static
Notify_value
ringgg (client, itimer_type)
	Notify_client	client;
	int		itimer_type;
{
	int		lunation;
	int		i, yy, mm, dd, hh, mmm, ss;
	int		aom_d, aom_h, aom_m;
	int		op;
	long		t;
	double		jd, p, aom, cphase, cdist, cangdia, csund, csuang;
	double		phasar [5];
	char		tbuf[80];
	struct tm	*gm;

	if (color_mode == TRUE) {
		op = PIX_SRC | PIX_COLOR(1);
	} else {
		op = PIX_NOT (PIX_SRC);
	}

	(void) time (&t);
	jd = jtime ((gm = gmtime (&t)));
	if (testmode) {
		static double	faketime = 0.0;

		if (faketime == 0.0) {
			faketime = jd + 1;
		} else {
			faketime += 1.0 / 24;
		}
		jd = faketime;
	}

	p = phase (jd, &cphase, &aom, &cdist, &cangdia, &csund, &csuang);
	aom_d = (int) aom;
	aom_h = (int) (24 * (aom - floor(aom)));
	aom_m = (int) (1440 * (aom - floor(aom))) % 60;

	drawmoon (p, virgin_mpr, icon_mpr);

	if ((int) window_get (frame, FRAME_CLOSED) == TRUE) {
		int		color;
		struct pr_prpos tloc;
		struct pr_size	txt_size;

		color = (color_mode == TRUE) ? 4 : 0;

		if (border == TRUE) {
			pr_vector (icon_mpr, 0, 0, 0, 63, PIX_SRC, color);
			pr_vector (icon_mpr, 0, 0, 63, 0, PIX_SRC, color);
			pr_vector (icon_mpr, 63, 0, 63, 63, PIX_SRC, color);
			pr_vector (icon_mpr, 0, 63, 63, 63, PIX_SRC, color);
		}

		if (aom_d == 0) {
                        (void)sprintf(tbuf, "%dh %dm", aom_h, aom_m);
		} else {
                        (void)sprintf(tbuf, "%dd %dh", aom_d, aom_h);
		}
		txt_size = pf_textwidth (strlen (tbuf), pfont, tbuf);
		tloc.pos.x = (64 - txt_size.x) / 2;
		tloc.pos.y = 63;
		tloc.pr = icon_mpr;
		pf_ttext (tloc, PIX_SRC | PIX_COLOR (color), pfont, tbuf);
		window_set (frame, FRAME_ICON, moon_icon, 0);

                /* If we're iconic, there's nothing more to do. */
		return (NOTIFY_DONE);
	}


	/* Update textual information for open window */

	/* start batching updates to the pixwin */
	pw_batch_on (cpw);

        (void)sprintf (tbuf, "%.5f", jd + 0.5); /* julian date */
	prt(1);

	if (testmode) { 			/* universal time */
		jyear (jd, &yy, &mm, &dd);
		jhms (jd, &hh, &mmm, &ss);
                (void)sprintf (tbuf, "%2d:%02d:%02d %2d %s %d            ",
			hh, mmm, ss, dd, moname [mm - 1], yy);
	} else {
                (void)sprintf (tbuf, "%2d:%02d:%02d %2d %s %d            ",
			gm->tm_hour, gm->tm_min, gm->tm_sec,
			gm->tm_mday, moname [gm->tm_mon], gm->tm_year + 1900);
	}
	prt(2);

	if (testmode == FALSE) {	/* Ignore local time in test mode */
		gm = localtime (&t);
		/* local time */
                (void)sprintf (tbuf, "%2d:%02d:%02d %2d %s %d           ",
			gm->tm_hour, gm->tm_min, gm->tm_sec,
			gm->tm_mday, moname [gm->tm_mon], gm->tm_year + 1900);
		prt(3);
	}


	/* Information about the Moon */

	/* age of moon */
        (void)sprintf (tbuf, "%d day%s, %d hour%s, %d minute%s.       ",
		EPL(aom_d), EPL(aom_h), EPL(aom_m));
	prt(5);

	/* moon phase */
        (void)sprintf (tbuf, "%d%%   (0%% = New, 100%% = Full)  ",
		(int) (cphase * 100));
	prt(6);

	/* moon distance */
        (void)sprintf (tbuf, "%ld kilometres, %.1f Earth radii.  ",
		(long) cdist, cdist / earthrad);
	prt(7);

	/* moon subtends */
        (void)sprintf (tbuf, "%.4f degrees.       ", cangdia);
	prt(8);

	/* Information about the Sun */

        /* sun's distance */
        (void)sprintf (tbuf, "%.0f kilometres, %.3f astronomical units.       ",
		csund, csund / sunsmax);
	prt(10);

	/* sun subtends */
        (void)sprintf (tbuf, "%.4f degrees.   ", csuang);
	prt(11);

        /* paint the moon's image in the upper right of the canvas */
	pw_rop (cpw, 58 * charwid, charhgt, 64, 64, PIX_SRC, icon_mpr, 0, 0);

	/* flush the pixwin updates to the screen */
	pw_batch_off (cpw);


	/*
	 * Calculate times of phases of this lunation.	This is sufficiently
	 * time-consuming that we only do it once a month.
	 */

	if (jd > nptime) {
		phasehunt(jd + 0.5, phasar);

		for (i = 0; i < 5; i++) {
			fmt_phase_time (phasar[i], tbuf);
			prt(APOS(i));
		}

		lunation = floor(((phasar[0] + 7) - lunatbase) / synmonth) + 1;
                (void)sprintf(tbuf, "Lunation %d ", lunation);
		prtxy(APOS(0), 49);			      
                (void)sprintf(tbuf, "Lunation %d ", lunation + 1);
		prtxy(APOS(4), 49);
		nptime = phasar[4];
	}

	return (NOTIFY_DONE);
}
#undef APOS


/*
 * FMT_PHASE_TIME -- Format the provided julian date into the provided buffer
 *		in UTC format for screen display
 */

fmt_phase_time (utime, buf)
	double	utime;
	char	*buf;
{
	int	yy, mm, dd, hh, mmm, ss;

	jyear (utime, &yy, &mm, &dd);
	jhms (utime, &hh, &mmm, &ss);
        (void)sprintf (buf, "%2d:%02d UTC %2d %s %d            ",
		hh, mmm, dd, moname [mm - 1], yy);
}


/*
 * PAINT_LABELS -- Draw the labels into the canvas (open) window and right
 *		justify them.  Done once at startup.  We paint the labels
 *		separately to minimize the amount of screen real-estate
 *		being modified on each update.	(Also lets us easily paint
 *		them in a different color).
 */

paint_labels ()
{
	int	i;

	for (i = 0; i < sizeof (labels) / sizeof (char *); i++) {
		if (color_mode == TRUE) {
			pw_ttext (cpw, charwid * (17 - strlen (labels [i])),
				charhgt * (i + 1), PIX_SRC | PIX_COLOR(4),
				NULL, labels [i]);
		} else {
			pw_text (cpw, charwid * (17 - strlen (labels [i])),
				charhgt * (i + 1), PIX_NOT (PIX_SRC),
				NULL, labels [i]);
		}
	}
}


/*
 * JDATE  --  Convert internal GMT date and time to Julian day
 *	       and fraction.
 */

static
long
jdate (t)
	struct tm	*t;
{
	long		c, m, y;

	y = t->tm_year + 1900;
	m = t->tm_mon + 1;
	if (m > 2) {
		m = m - 3;
	} else {
		m = m + 9;
		y--;
	}
	c = y / 100L;		   /* Compute century */
	y -= 100L * c;
	return (t->tm_mday + (c * 146097L) / 4 + (y * 1461L) / 4 +
	    (m * 153L + 2) / 5 + 1721119L);
}


/*
 * JTIME --    Convert internal GMT date and time to astronomical Julian
 *	       time (i.e. Julian date plus day fraction, expressed as
 *	       a double).
 */

static
double
jtime (t)
	struct tm *t;
{
	return (jdate (t) - 0.5) + 
	   (t->tm_sec + 60 * (t->tm_min + 60 * t->tm_hour)) / 86400.0;
}


/*
 * JYEAR  --  Convert Julian date to year, month, day, which are
 *	       returned via integer pointers to integers.  
 */

static
void
jyear (td, yy, mm, dd)
	double	td;
	int	*yy, *mm, *dd;
{
	double j, d, y, m;

	td += 0.5;				/* Astronomical to civil */
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
	if (m < 10.0)
		m = m + 3;
	else {
		m = m - 9;
		y = y + 1;
	}
	*yy = y;
	*mm = m;
	*dd = d;
}


/*
 * JHMS  --  Convert Julian time to hour, minutes, and seconds.
 */

static
void
jhms(j, h, m, s)
	double j;
	int *h, *m, *s;
{
	long ij;

	j += 0.5;				/* Astronomical to civil */
	ij = (j - floor(j)) * 86400.0;
	*h = ij / 3600L;
	*m = (ij / 60L) % 60L;
	*s = ij % 60L;
}


/*
 * MEANPHASE  --  Calculates time of the mean new Moon for a given
 *		base date.  This argument K to this function is
 *		the precomputed synodic month index, given by:
 *
 *			K = (year - 1900) * 12.3685
 *
 *		where year is expressed as a year and fractional year.
 */

static
double
meanphase (sdate, k)
	double	sdate, k;
{
	double	t, t2, t3, nt1;

	/* Time in Julian centuries from 1900 January 0.5 */
	t = (sdate - 2415020.0) / 36525;
	t2 = t * t;			/* Square for frequent use */
	t3 = t2 * t;			/* Cube for frequent use */

	nt1 = 2415020.75933 + synmonth * k
		+ 0.0001178 * t2
		- 0.000000155 * t3
		+ 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

	return nt1;
}


/*
 * TRUEPHASE  --  Given a K value used to determine the
 *		mean phase of the new moon, and a phase
 *		selector (0.0, 0.25, 0.5, 0.75), obtain
 *		the true, corrected phase time.
 */

static
double
truephase(k, phase)
	double k, phase;
{
	double t, t2, t3, pt, m, mprime, f;
	int apcor = FALSE;

	k += phase;		   /* Add phase to new moon time */
	t = k / 1236.85;	   /* Time in Julian centuries from
				      1900 January 0.5 */
	t2 = t * t;		   /* Square for frequent use */
	t3 = t2 * t;		   /* Cube for frequent use */
	pt = 2415020.75933	   /* Mean time of phase */
	     + synmonth * k
	     + 0.0001178 * t2
	     - 0.000000155 * t3
	     + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

        m = 359.2242               /* Sun's mean anomaly */
	    + 29.10535608 * k
	    - 0.0000333 * t2
	    - 0.00000347 * t3;
        mprime = 306.0253          /* Moon's mean anomaly */
	    + 385.81691806 * k
	    + 0.0107306 * t2
	    + 0.00001236 * t3;
        f = 21.2964                /* Moon's argument of latitude */
	    + 390.67050646 * k
	    - 0.0016528 * t2
	    - 0.00000239 * t3;
	if ((phase < 0.01) || (abs(phase - 0.5) < 0.01)) {

	   /* Corrections for New and Full Moon */

	   pt +=     (0.1734 - 0.000393 * t) * dsin(m)
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
	   pt +=     (0.1721 - 0.0004 * t) * dsin(m)
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
	   (void)fprintf (stderr,
                "TRUEPHASE called with invalid phase selector.\n");
	   abort();
	}
	return pt;
}


/*
 * PHASEHUNT  --  Find time of phases of the moon which surround
 *		the current date.  Five phases are found, starting
 *		and ending with the new moons which bound the
 *		current lunation.
 */

static
void
phasehunt (sdate, phases)
	double	sdate;
	double	phases [5];
{
	double	adate, k1, k2, nt1, nt2;
	int	yy, mm, dd;

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
	phases [0] = truephase (k1, 0.0);
	phases [1] = truephase (k1, 0.25);
	phases [2] = truephase (k1, 0.5);
	phases [3] = truephase (k1, 0.75);
	phases [4] = truephase (k2, 0.0);
}


/*
 * KEPLER  --	Solve the equation of Kepler.
 */

static
double
kepler(m, ecc)
	double m, ecc;
{
	double e, delta;
#define EPSILON 1E-6

	e = m = torad(m);
	do {
		delta = e - ecc * sin(e) - m;
		e -= delta / (1 - ecc * cos(e));
	} while (abs (delta) > EPSILON);
	return e;
}


/*
 * PHASE  --  Calculate phase of moon as a fraction:
 *
 *	The argument is the time for which the phase is requested,
 *	expressed as a Julian date and fraction.  Returns the terminator
 *	phase angle as a percentage of a full circle (i.e., 0 to 1),
 *	and stores into pointer arguments the illuminated fraction of
 *      the Moon's disc, the Moon's age in days and fraction, the
 *	distance of the Moon from the centre of the Earth, and the
 *	angular diameter subtended by the Moon as seen by an observer
 *	at the centre of the Earth.
 */

static
double
phase (pdate, pphase, mage, dist, angdia, sudist, suangdia)
	double	pdate;
	double	*pphase;		/* Illuminated fraction */
	double	*mage;			/* Age of moon in days */
	double	*dist;			/* Distance in kilometres */
	double	*angdia;		/* Angular diameter in degrees */
	double	*sudist;		/* Distance to Sun */
        double  *suangdia;              /* Sun's angular diameter */
{

	double	Day, N, M, Ec, Lambdasun, ml, MM, MN, Ev, Ae, A3, MmP,
		mEc, A4, lP, V, lPP, NP, y, x, Lambdamoon, BetaM,
		MoonAge, MoonPhase,
		MoonDist, MoonDFrac, MoonAng, MoonPar,
		F, SunDist, SunAng;

        /* Calculation of the Sun's position */

	Day = pdate - epoch;			/* Date within epoch */
	N = fixangle((360 / 365.2422) * Day);	/* Mean anomaly of the Sun */
	M = fixangle(N + elonge - elongp);	/* Convert from perigee
						co-ordinates to epoch 1980.0 */
	Ec = kepler(M, eccent); 		/* Solve equation of Kepler */
	Ec = sqrt((1 + eccent) / (1 - eccent)) * tan(Ec / 2);
	Ec = 2 * todeg(atan(Ec));		/* True anomaly */
        Lambdasun = fixangle(Ec + elongp);      /* Sun's geocentric ecliptic
							longitude */
	/* Orbital distance factor */
	F = ((1 + eccent * cos(torad(Ec))) / (1 - eccent * eccent));
	SunDist = sunsmax / F;			/* Distance to Sun in km */
        SunAng = F * sunangsiz;         /* Sun's angular size in degrees */


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
