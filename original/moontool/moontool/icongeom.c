/*

	Process -icongeometry declaration from a command line and
	delete it from the argument string.

	If an error is detected, a pointer to an ASCII error message
	is returned; otherwise NULL is returned.

*/

#include <X11/Intrinsic.h>
#include <X11/Xutil.h>
#include <X11/Shell.h>

char *icongeom(toplevel, argc, argv)
  Widget toplevel;
  int *argc;
  char *argv[];
{
    int i, j, mask;
    int icongeomspec = FALSE;
    int x, y;
    unsigned int width, height;

    for (i = 1; i < *argc; i++) {
        if (strcmp(argv[i], "-icongeometry") == 0) {

	    if (icongeomspec) {
                return "duplicate -icongeometry specification";
	    }
	    icongeomspec = TRUE;
	    if ((i + 1) >= *argc) {
                return "missing argument after -icongeometry";
	    }
	    mask = XParseGeometry(argv[i + 1], &x, &y, &width, &height);
	    if (mask == 0) {
                return "bad argument to -icongeometry";
	    }

	    /* Delete option and argument from argument list. */

	    for (j = i + 2; j < *argc; j++) {
		argv[j - 2] = argv[j];
	    }
	    *argc -= 2;
	    i--;		      /* Compensate for increment at loop end */

	    if (mask & XValue) {
		XtVaSetValues(toplevel,
		    XtNiconX, (XtArgVal) x,
		    NULL);
	    }
	    if (mask & YValue) {
		XtVaSetValues(toplevel,
		    XtNiconY, (XtArgVal) y,
		    NULL);
	    }
	}
    }
    return NULL;
}
