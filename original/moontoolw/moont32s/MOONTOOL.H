/*

        Moontool for Windows

            Definitions

*/

#include <windows.h>
#include <shellapi.h>
#include <string.h>
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

#include "resource.h"

#define Format(n)   rstring(IDS_FORMATS + (n))

struct moongeom {
    short winwid;                           /* Window width */
    short winhgt;                           /* Window height */
    short moonbitx;                         /* Moon bitmap character column */
    short infocol;                          /* Information column */
    short luncol;                           /* Lunation column */
};

extern struct moongeom mgeom;               /* Moontool geometry definition */

extern char szString[128];   /* Variable to load resource strings */
extern char szAppName[20];   /* Class name for the window */
extern HINSTANCE hInst;      /* Class instance pointer */
extern HWND hWndMain;        /* Main window pointer */

extern LRESULT CALLBACK WndProc(HWND hWnd, UINT Message, WPARAM wParam, LPARAM lParam);

/* Dialogue service functions. */

extern BOOL FAR PASCAL About(HWND, UINT, WPARAM, LPARAM);
extern BOOL FAR PASCAL SetJdate(HWND, UINT, WPARAM, LPARAM);
extern BOOL FAR PASCAL SetUtime(HWND, UINT, WPARAM, LPARAM);

/*  Calculation functions from MOONCALC.C  */

extern TIME_ZONE_INFORMATION tzInfo;		// Time zone information
extern HICON moontoolIcon;					/* Application icon */
extern NOTIFYICONDATA trayIcon;				// Taskbar tray icon
extern BOOL inTray;							// Shown in system tray ?

extern void set_tm_time(struct tm *t, BOOL islocal);
extern double jtime(struct tm *t);
extern void jyear(double td, long *yy, int *mm, int *dd),
            jhms(double j, int *h, int *m, int *s);
extern int jwday(double j);
extern double gmst(double jd);
extern double ucttoj(long year, int mon, int mday,
                     int hour, int min, int sec);
extern double phase(double  pdate, double  *pphase, double  *mage,
                    double  *dist, double  *angdia, double  *sudist,
                    double  *suangdia);
extern void phasehunt(double sdate, double phases[5]),
            fmt_phase_time(double utime, char *buf);

extern void ringgg(HWND hWnd, HDC hDC, int repaint);
extern void paint_labels(HDC hDC);
extern void go_iconic(void);
extern char *rstring(int resid);

extern int testmode, runmode;
extern double faketime;
