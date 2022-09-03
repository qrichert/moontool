/*

        Moontool for Windows

            Definitions

*/

#include <windows.h>
#include <string.h>
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

#include "julian.h"
#include "utc.h"

#define IDM_ABOUT   100
#define IDM_TIMEZONE              5000      /* Time zone dialogue */

#define IDM_FILE                  1000
#define IDM_F_EXIT                     1050
#define IDM_EDIT                  2000
#define IDM_F_COPY                     2050
#define IDM_OPTIONS               3000
#define IDM_F_TESTMODE                 3050
#define IDM_F_STOP                     3060
#define IDM_F_RUN                      3070
#define IDM_F_SETJDATE                 3080
#define IDM_F_SETUTIME                 3090
#define IDM_HELP                  4000
#define IDM_F_ABOUT                    4050
#define IDM_F_TIMEZONE                 4060

#define IDS_ERR_REGISTER_CLASS   1
#define IDS_ERR_CREATE_WINDOW    2
#define IDS_ERR_TRUEPHASE        3
#define IDS_ERR_IERR             4
#define IDS_ERR_TZWARN           5          /* Time zone warning caption */
#define IDS_ERR_NOTZSET          6          /* No time zone set */
#define IDS_ITEM_LABELS         16          /* Item label base */
#define IDS_ITEM_LABELS_N       17
#define IDS_MONTH_NAMES         48          /* Month name base */
#define IDS_FORMATS             64          /* Format base */

#define IDS_MINUTE              96          /* Time labels: plurals must be */
#define IDS_MINUTES             97          /* singular code plus 1. */
#define IDS_HOUR                98
#define IDS_HOURS               99
#define IDS_DAY                100
#define IDS_DAYS               101

#define IDS_APPNAME            102          /* Application name/title */

#define Format(n)   rstring(IDS_FORMATS + (n))

struct moongeom {
    int winwid;                             /* Window width */
    int winhgt;                             /* Window height */
    int moonbitx;                           /* Moon bitmap character column */
    int infocol;                            /* Information column */
    int luncol;                             /* Lunation column */
};

extern struct moongeom mgeom;               /* Moontool geometry definition */

extern char szString[128];   /* Variable to load resource strings */
extern char szAppName[20];   /* Class name for the window */
extern HWND hInst;           /* Class instance pointer */
extern HWND hWndMain;        /* Main window pointer */


LONG FAR PASCAL WndProc(HWND, WORD, WORD, LONG);
int nCwRegisterClasses(void);
#ifdef UNREGISTER_CLASSES_NEEDED
void CwUnRegisterClasses(void);
#endif

/* Dialogue service functions. */

BOOL FAR PASCAL About(HWND, unsigned, WORD, LONG);
BOOL FAR PASCAL SetJdate(HWND, unsigned, WORD, LONG);
BOOL FAR PASCAL SetUtime(HWND, unsigned, WORD, LONG);

/*  Calculation functions from MOONCALC.C  */

extern double jtime(struct tm *t),
              phase(double  pdate, double  *pphase, double  *mage,
                    double  *dist, double  *angdia, double  *sudist,
                    double  *suangdia);
extern void phasehunt(double sdate, double phases[5]),
            fmt_phase_time(double utime, char *buf);
extern void jyear(double td, long *yy, int *mm, int *dd),
            jhms(double j, int *h, int *m, int *s);
extern long jdate(struct tm *t);
extern double ucttoj(long year, int mon, int mday,
                     int hour, int min, int sec);
extern void ringgg(HWND hWnd, HDC hDC, int repaint);
extern void paint_labels(HDC hDC);
extern void go_iconic(void);
extern char *rstring(int resid);

extern int testmode, runmode;
extern double faketime;
