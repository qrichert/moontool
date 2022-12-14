/*

                          Moontool for Windows

                    Astronomical Calculation Routines

*/

#include "moontool.h"

/*  Astronomical constants  */

#define epoch       2444238.5      /* 1980 January 0.0 */

/*  Constants defining the Sun's apparent orbit  */

#define elonge      278.833540     /* Ecliptic longitude of the Sun
                                      at epoch 1980.0 */
#define elongp      282.596403     /* Ecliptic longitude of the Sun at
                                      perigee */
#define eccent      0.016718       /* Eccentricity of Earth's orbit */
#define sunsmax     1.495985e8     /* Semi-major axis of Earth's orbit, km */
#define sunangsiz   0.533128       /* Sun's angular size, degrees, at
                                      semi-major axis distance */

/*  Elements of the Moon's orbit, epoch 1980.0  */

#define mmlong      64.975464      /* Moon's mean longitude at the epoch */
#define mmlongp     349.383063     /* Mean longitude of the perigee at the
                                      epoch */
#define mlnode      151.950429     /* Mean longitude of the node at the
                                      epoch */
#define minc        5.145396       /* Inclination of the Moon's orbit */
#define mecc        0.054900       /* Eccentricity of the Moon's orbit */
#define mangsiz     0.5181         /* Moon's angular size at distance a
                                      from Earth */
#define msmax       384401.0       /* Semi-major axis of Moon's orbit in km */
#define mparallax   0.9507         /* Parallax at distance a from Earth */
#define synmonth    29.53058868    /* Synodic month (new Moon to new Moon) */
#define lunatbase   2423436.0      /* Base date for E. W. Brown's numbered
                                      series of lunations (1923 January 16) */

/*  Properties of the Earth  */

#define earthrad    6378.16        /* Radius of Earth in kilometres */

#define PI 3.14159265358979323846  /* Assume not near black hole nor in
                                      Tennessee */

/*  Handy mathematical functions  */

#define sgn(x) (((x) < 0) ? -1 : ((x) > 0 ? 1 : 0))       /* Extract sign */
#define abs(x) ((x) < 0 ? (-(x)) : (x))                   /* Absolute val */
#define fixangle(a) ((a) - 360.0 * (floor((a) / 360.0)))  /* Fix angle    */
#define torad(d) ((d) * (PI / 180.0))                     /* Deg->Rad     */
#define todeg(d) ((d) * (180.0 / PI))                     /* Rad->Deg     */
#define dsin(x) (sin(torad((x))))                         /* Sin from deg */
#define dcos(x) (cos(torad((x))))                         /* Cos from deg */

int testmode = FALSE;                 /* Rapid warp through time for debugging */
int runmode = TRUE;                   /* Update time if true */
double faketime = 0.0;                /* Time increment for test mode */
static int color_mode = FALSE;        /* Indicates color/mono mode */
static double nptime = 0.0;           /* Next new moon time */
TIME_ZONE_INFORMATION tzInfo;		  // Time zone information

#ifdef NEEDED
static void set_system_time(SYSTEMTIME *s, struct tm *t);
#endif

static char olabel[IDS_ITEM_LABELS_N][60]; /* Old label values */
static char luabel[IDS_ITEM_LABELS_N][60]; /* Old lunation values */
static char last_icon_tag[64];        /* Old icon label */
static SIZE olext[IDS_ITEM_LABELS_N]; /* Old label text extent */
static SIZE luext[IDS_ITEM_LABELS_N]; /* Old lunation extent */

/*  RSTRING  --  Retrieve a string from the resource file.  */

char *rstring(int resid)
{
#define maxCStrings 10              /* Maximum concurrently used strings */
    static char rstrings[maxCStrings][80];
    static int n = 0;
    int m = n;

    if (LoadString(hInst, resid, rstrings[m], 79) < 0) {
        strcpy(rstrings[m], "");
    }
    n = (n + 1) % maxCStrings;
    return rstrings[m];
}

/*	UPDATEICON  --  Update tray icon.  Watch out!  This code is
					not general but rather knows that the resource
					bitmap it works on is 4 bits per pixel, uncompressed.  */

static void updateIcon(HWND hWnd, HDC hDC, char *bmpname, int width, int CENTER,
					  int IRADIUS, int OFFSET,
                      int xpos, int ypos, double ph, int mm, int dd)
{
	HRSRC rsc;
	HGLOBAL bitmap;
	LPBITMAPINFOHEADER b, ib, ab;
	ICONINFO ici;
	HICON newIcon;
    int i, x, lx, rx;
    double cp, xscale, RADIUS = IRADIUS;

	if ((rsc = FindResource(hInst, bmpname, RT_BITMAP)) != NULL) {
		if ((bitmap = LoadResource(hInst, rsc)) != NULL) {
			int bp, is, l, llen;

			b = (LPBITMAPINFOHEADER) bitmap;
			bp = (b->biClrUsed != 0) ? b->biClrUsed : (1 << b->biBitCount);
			llen = ((((b->biWidth * b->biBitCount) / 8) + 3) & (~3));
			is = (b->biSizeImage != 0) ? b->biSizeImage : (llen * b->biHeight);
			l = b->biSize + bp * sizeof(RGBQUAD) + is;
			if ((ib = malloc(l * 2)) != NULL) {
				HBITMAP bm1, bm2;
				LPSTR pixels;

				//	First copy of DIB is for image

				memcpy(ib, b, l);
				ab = (LPBITMAPINFOHEADER) (((LPSTR) ib) + l);

				//	Create second copy of DIB for AND mask and clear it

				memcpy(ab, b, l - is);
				memset(((LPSTR) ab) + (ab->biSize + bp * sizeof(RGBQUAD)), 0, is);

				//	Now create the device-dependent bitmaps used to build the icon

				bm1 = CreateCompatibleBitmap(hDC, b->biWidth, b->biHeight);
				bm2 = CreateCompatibleBitmap(hDC, b->biWidth, b->biHeight);

				pixels = ((LPSTR) ib) + (ib->biSize + bp * sizeof(RGBQUAD));

#define SetPix(x, y, c) {	int mask = 0xF0 >> (4 * ((x) & 1)); \
							LPSTR pix = pixels + (llen * ((b->biHeight - 1) - (y))) + \
									((x) >> 1); \
							*pix = ((*pix) & mask) | \
									(((c) | ((c) << 4))) & (~mask); \
						}

				if ((color_mode == TRUE) && ((ph < 0.01) || (ph > 0.99))) {
					memset(((LPSTR) ib) + (ib->biSize + bp * sizeof(RGBQUAD)), 0, is);
				} else {
					xscale = cos(2 * PI * ph);
					for (i = 0; i < IRADIUS; i++) {
						cp = RADIUS * cos(asin((double) i / RADIUS));
						if (ph < 0.5) {
							rx = (int) (CENTER + cp);
							lx = (int) (CENTER + xscale * cp);
						} else {
							lx = (int) (CENTER - cp);
							rx = (int) (CENTER - xscale * cp);
						}

						/* We now know the left and right endpoints of the scan line
						   for this y  coordinate.  Clear black region of each line
						   in XOR DIB prior to copying to DDB.  */

						if (lx > 0) {
							for (x = 0; x <= lx; x++) {
								SetPix(x, OFFSET + i, 0);
							}
							if (rx < width) {
								for (x = 0; x <= width - rx; x++) {
									SetPix(rx + x, OFFSET + i, 0);
								}
							}
							if (i != 0) {
								for (x = 0; x <= lx; x++) {
									SetPix(x, OFFSET - i, 0);
								}
								if (rx < 64) {
									for (x = 0; x <= width - rx; x++) {
										SetPix(rx + x, OFFSET - i, 0);
									}
								}
							}
						}
					}
				}

				//	Set the DIBs to the DDBs

				SetDIBits(hDC, bm1, 0, ab->biHeight,
						  ((LPCSTR) ab) + (ab->biSize + bp * sizeof(RGBQUAD)),
						  (LPBITMAPINFO) ab, DIB_RGB_COLORS);
				SetDIBits(hDC, bm2, 0, ib->biHeight,
						  ((LPCSTR) ib) + (ib->biSize + bp * sizeof(RGBQUAD)),
						  (LPBITMAPINFO) ib, DIB_RGB_COLORS);

				ici.fIcon = TRUE;
				ici.xHotspot = ici.yHotspot = 0;
				ici.hbmMask = bm1;
				ici.hbmColor = bm2;
				newIcon = CreateIconIndirect(&ici);

				if (newIcon != NULL) {
					if (moontoolIcon != NULL) {
						DestroyIcon(moontoolIcon);
					}
					moontoolIcon = newIcon;
				}

				if (inTray) {
					trayIcon.hIcon = moontoolIcon;
					Shell_NotifyIcon(NIM_MODIFY, &trayIcon);
				}

				DeleteObject(bm1);
				DeleteObject(bm2);
				free(ib);
			}
			FreeResource(bitmap);
		}
	}
}

/*  DRAWMOON  --  Construct icon for moon, given phase of moon.  */

static void drawmoon(HDC hDC, char *bmpname, int width, int CENTER,
                     int IRADIUS, int OFFSET,
                     int xpos, int ypos, double ph, int mm, int dd)
{
    int i, lx, rx;
    double cp, xscale, RADIUS = IRADIUS;
    static char tbuf[20];

    HDC hMemoryDC;
    HBITMAP hColourmoon;

	if ((GetDeviceCaps(hDC, BITSPIXEL) <= 8) &&
		(strcmp(bmpname, "COLOURMOON") == 0)) {
		bmpname = "GREYMOON";
	}

    hColourmoon = LoadBitmap(hInst, bmpname);

    hMemoryDC = CreateCompatibleDC(hDC);
    SelectObject(hMemoryDC, hColourmoon);

    /* If it's July 20th (in local time if we're running in real time,
       otherwise based on UTC), display the  Apollo  11  Commemorative
       Red  Dot at Tranquility Base.  Otherwise, just show the regular
       mare floor.  */

    if ((mm == 7) && (dd == 20) && (strcmp(bmpname, "COLOURMOON") == 0)) {
        SetPixel(hMemoryDC, 41, 29, RGB(255, 0, 0));
    }

    if ((color_mode == TRUE) && ((ph < 0.01) || (ph > 0.99))) {
        return;
    }

    xscale = cos(2 * PI * ph);
    for (i = 0; i < IRADIUS; i++) {
        cp = RADIUS * cos(asin((double) i / RADIUS));
        if (ph < 0.5) {
            rx = (int) (CENTER + cp);
            lx = (int) (CENTER + xscale * cp);
        } else {
            lx = (int) (CENTER - cp);
            rx = (int) (CENTER - xscale * cp);
        }

        /* We  now know the left and right  endpoints of the scan line
           for this y  coordinate.   We  raster-op  the  corresponding
           scanlines  from  the  source  pixrect  to  the  destination
           pixrect, offsetting to properly place it in the pixrect and
           reflecting vertically. */

        if (lx > 0) {
            BitBlt(hMemoryDC, 0, OFFSET + i, lx, 1, NULL, 0, 0, BLACKNESS);
            if (rx < width) {
                BitBlt(hMemoryDC, rx, OFFSET + i, width - rx, 1, NULL, 0, 0, BLACKNESS);
            }
            if (i != 0) {
                BitBlt(hMemoryDC, 0, OFFSET - i, lx, 1, NULL, 0, 0, BLACKNESS);
                if (rx < 64) {
                    BitBlt(hMemoryDC, rx, OFFSET - i, width - rx, 1, NULL, 0, 0, BLACKNESS);
                }
            }
        }
    }

    BitBlt(hDC, xpos, ypos, width == 64 ? 58 : width,
           width == 64 ? 58 : width, hMemoryDC,
           width == 64 ? 2 : 0, 0, SRCCOPY);
    DeleteDC(hMemoryDC);
    DeleteObject(hColourmoon);
}

/*  PAINT_LABELS  --  Paint item labels in window.  */

void paint_labels(HDC hDC)
{
    int i, line_space;
    TEXTMETRIC tm;

    GetTextMetrics(hDC, &tm);
    line_space = tm.tmHeight + tm.tmExternalLeading;

    for (i = 0; i < IDS_ITEM_LABELS_N; i++) {
        char labtext[40];

        if (LoadString(hInst, IDS_ITEM_LABELS + i, labtext, (sizeof labtext) - 1) > 0) {
            TextOut(hDC, 3, i * line_space, labtext, strlen(labtext));
        }
    }
}

/*  GO_ICONIC  --  Initialise when window is minimised to an icon. */

void go_iconic(void)
{
    strcpy(last_icon_tag, "");              /* Force icon tag redisplay */
}

/*	openSystemTimeToTzSpecificLocalTime  --  Re-implementation of NT-only
											 time zone conversion function the
											 code kiddies could be bothered to
											 include in other Win32 platforms.
											 Fuck 'em.  Note also that due to
											 the idiot epoch for FILETIME (January 1,
											 1601 Gregorian, 00:00 UTC), we punt on
											 dates prior to that.  */

static BOOL openSystemTimeToTzSpecificLocalTime(LPTIME_ZONE_INFORMATION itz,
												LPSYSTEMTIME ut, LPSYSTEMTIME lt)
{
	TIME_ZONE_INFORMATION tz;
	FILETIME now;
	LONGLONG q_now;
	LONG currentBias;

	if ((short) (ut->wYear) < 1601) {
		return FALSE;
	}

	tz = *itz;
	currentBias = tz.Bias;
	SystemTimeToFileTime(ut, &now);
	memcpy(&q_now, &now, sizeof(q_now));

	/*	Now, if this time zone definition includes transition to
		and from summer time, undertake the massive flailing around
		Windows makes us do to determine the correct bias for the
		current date.  */

	if (tz.DaylightDate.wMonth != 0 && tz.StandardDate.wMonth != 0) {
		FILETIME dstStart, dstEnd;
		LONGLONG q_dstStart, q_dstEnd;
		double jd;
		int nday, mm, dd;
		long yy;
		BOOL south, isDaylight;

		/*	If the time zone is in the "nth day in month" form, obtain
			the date for the given year by searching for the first
			occurrence of the given weekday in the month, then advancing
			by weeks until the given nth day is reached.  */

		if (tz.DaylightDate.wYear == 0) {
			tz.DaylightDate.wYear = ut->wYear;
			jd = ucttoj(tz.DaylightDate.wYear, tz.DaylightDate.wMonth - 1, 1, 12, 0, 0);
			while (jwday(jd) != tz.DaylightDate.wDayOfWeek) {
				jd++;		
			}
			nday = 1;
			while (nday < tz.DaylightDate.wDay) {
				jd += 7;
				jyear(jd, &yy, &mm, &dd);
				if (mm != tz.DaylightDate.wMonth) {
					jd -= 7;
					break;
				}
				nday++;
			}
		}
		jyear(jd, &yy, &mm, &dd);
		tz.DaylightDate.wDay = dd;

		//	Now do the same thing for the start of standard time

		if (tz.StandardDate.wYear == 0) {
			tz.StandardDate.wYear = ut->wYear;
			jd = ucttoj(tz.StandardDate.wYear, tz.StandardDate.wMonth - 1, 1, 12, 0, 0);
			while (jwday(jd) != tz.StandardDate.wDayOfWeek) {
				jd++;		
			}
			nday = 1;
			while (nday < tz.StandardDate.wDay) {
				jd += 7;
				jyear(jd, &yy, &mm, &dd);
				if (mm != tz.StandardDate.wMonth) {
					jd -= 7;
					break;
				}
				nday++;
			}
		}
		jyear(jd, &yy, &mm, &dd);
		tz.StandardDate.wDay = dd;

		/*	Now that we have absolute starting and ending dates
			for the given year, set up to compare the given date
			to see which period it falls into.  */

		SystemTimeToFileTime(&(tz.DaylightDate), &dstStart);
		SystemTimeToFileTime(&(tz.StandardDate), &dstEnd);
		memcpy(&q_dstStart, &dstStart, sizeof(q_dstStart));
		memcpy(&q_dstEnd, &dstEnd, sizeof(q_dstEnd));

		/*	If the end of summer time precedes its starting date,
			we're probably in the southern hemisphere.  Set the
			"south" flag in indicate the sense of the comparison
			is reversed and swap the starting and ending dates.  */

		south = (CompareFileTime(&dstEnd, &dstStart) < 0) ? 1 : 0;
		if (south) {
			LONGLONG t = q_dstStart;

			q_dstStart = q_dstEnd;
			q_dstEnd = t;
		}
		isDaylight = ((CompareFileTime(&now, &dstStart) > 0) &&
					 (CompareFileTime(&now, &dstEnd) <= 0));
		if (south) {
			isDaylight = !isDaylight;
		}

		//	Finally, adjust the bias to the value appropriate to this date

		currentBias += isDaylight ? tz.DaylightBias : tz.StandardBias;
	}

	/*	Now that we have the proper bias for this date, perhaps
		adjusted due to summer time, use it to calculate the
		file time number for the local time zone, then convert
		that back into a system time structure.  */

	q_now -= currentBias * 60 * 10000000i64;	// "File time" is in units of 100 ns

	/*	Even though we tested for the epoch at the top, we need to
		test again because application of the bias may have moved
		the date before the start of the epoch.  */

	if (q_now < 0) {
		return FALSE;
	}		
	memcpy(&now, &q_now, sizeof now);
	FileTimeToSystemTime(&now, lt);

	return TRUE;
}

/*  RINGGG  --  Update  status  on  interval  timer  ticks  and redraw
                window if needed.  */

#define prt(y) if (repaint || strcmp(olabel[y - 1], tbuf) != 0) { \
        if (olext[y - 1].cx != 0 || olext[y - 1].cy != 0) { \
              PatBlt(hDC, info_col, (y - 1) * lineSpace, \
              olext[y - 1].cx, olext[y - 1].cy, PATCOPY); \
        } \
        GetTextExtentPoint32(hDC, tbuf, strlen(tbuf), &olext[y - 1]); \
        TextOut(hDC, info_col, (y - 1) * lineSpace, tbuf, strlen(tbuf)); \
        strcpy(olabel[y - 1], tbuf); }

#define prl(y) if (repaint || strcmp(luabel[y - 1], tbuf) != 0) { \
        if (luext[y].cx != 0 || luext[y].cy != 0) { \
            PatBlt(hDC, loonie_col, (y - 1) * lineSpace, \
                        luext[y].cx, luext[y].cy, PATCOPY); \
        } \
        GetTextExtentPoint32(hDC, tbuf, strlen(tbuf), &luext[y]); \
        TextOut(hDC, loonie_col, (y - 1) * lineSpace, tbuf, strlen(tbuf)); \
        strcpy(luabel[y - 1], tbuf); }

#define Plural(x, n) (x), (LPSTR) rstring(n + ((x) == 1 ? 0 : 1))
#define APOS(x) (x + 13)

void ringgg(HWND hWnd, HDC hDC, int repaint)
{
    int lunation;
    int i, yy, mm, dd, hh, mmm, ss;
    int aom_d, aom_h, aom_m;
    double jd, p, aom, cphase, cdist, cangdia, csund, csuang;
    double phasar[5];
    char tbuf[80];
    struct tm gm;
    TEXTMETRIC tm;
    int lineSpace, info_col, loonie_col;
    HBRUSH hBrush, oBrush;

    GetTextMetrics(hDC, &tm);
    lineSpace = tm.tmHeight + tm.tmExternalLeading;
    info_col = tm.tmMaxCharWidth * mgeom.infocol;
    loonie_col = info_col + tm.tmMaxCharWidth * mgeom.luncol;

    if (testmode) {
        if (runmode && !repaint) {
            faketime += 1.0 / 24;
        }
        jd = faketime;
        jyear(jd, &yy, &mm, &dd);
    } else {
        if (runmode) {
			set_tm_time(&gm, FALSE);
			faketime = jd = jtime(&gm);
            mm = gm.tm_mon + 1;
            dd = gm.tm_mday;
        } else {
            jd = faketime;
            jyear(jd, &yy, &mm, &dd);
        }
    }

    p = phase(jd, &cphase, &aom, &cdist, &cangdia, &csund, &csuang);
    aom_d = (int) aom;
    aom_h = (int) (24 * (aom - floor(aom)));
    aom_m = (int) (1440 * (aom - floor(aom))) % 60;

    if (IsIconic(hWnd)) {

        /* Iconic */

        if (aom_d == 0) {
			wsprintf((LPSTR) tbuf, (LPSTR) Format(0),
					Plural(aom_h, IDS_HOUR),
					Plural(aom_m, IDS_MINUTE));
        } else {
			wsprintf((LPSTR) tbuf, (LPSTR) Format(0),
					Plural(aom_d, IDS_DAY),
					Plural(aom_h, IDS_HOUR));

        }
        if (strcmp(last_icon_tag, tbuf) != 0) {
            strcpy(last_icon_tag, tbuf);
            SetWindowText(hWndMain, tbuf);
			strcpy(trayIcon.szTip, tbuf);
        }
        drawmoon(hDC, "ICONMOON", 32, /* X ctr */ 16, /* X radius */ 14,
                 /* Y centre */ 14, 2, 2, p, mm, dd);
        updateIcon(hWnd, hDC, "ICONMOON", 32, /* X ctr */ 16, /* X radius */ 14,
                 /* Y centre */ 14, 2, 2, p, mm, dd);
        return;
    }

    hBrush = CreateSolidBrush(GetSysColor(COLOR_WINDOW));
    oBrush = SelectObject(hDC, hBrush);
    drawmoon(hDC, "COLOURMOON", 64, 32, 27, 28,
             tm.tmMaxCharWidth * mgeom.moonbitx, 10, p, mm, dd);

    /* Update textual information for open window */

    sprintf(tbuf, Format(2), jd); /* Julian date */
    prt(1);

    if (testmode || !runmode) {         /* Universal time */
        jyear(jd, &yy, &mm, &dd);
        jhms(jd, &hh, &mmm, &ss);
        wsprintf((LPSTR) tbuf, (LPSTR) Format(3),
                hh, mmm, ss, dd,
                (LPSTR) rstring(IDS_MONTH_NAMES + (mm - 1)), yy);
    } else {
        wsprintf((LPSTR) tbuf, (LPSTR) Format(4),
                gm.tm_hour, gm.tm_min, gm.tm_sec,
                gm.tm_mday,
                (LPSTR) rstring(IDS_MONTH_NAMES + gm.tm_mon), gm.tm_year + 1900);
    }
    prt(2);

    if (testmode || !runmode) {  /* Ignore local time in test mode */
		SYSTEMTIME s, local;

		s.wYear = yy;
		s.wMonth = mm;
		s.wDay = dd;
		s.wDayOfWeek = jwday(jd);
		s.wHour = hh;
		s.wMinute = mmm;
		s.wSecond = ss;
		s.wMilliseconds = 0;
		if (SystemTimeToTzSpecificLocalTime(&tzInfo, &s, &local)) {
			wsprintf((LPSTR) tbuf, (LPSTR) Format(4),
					local.wHour, local.wMinute, local.wSecond,
					local.wDay,
					(LPSTR) rstring(IDS_MONTH_NAMES + (local.wMonth - 1)), local.wYear);
		} else {
			/*	Code kiddies couldn't be bothered to implement
				SystemTimeToTzSpecificLocalTime on anything but
				NT, so to Hell with them--use our own implementation.  */
			if (openSystemTimeToTzSpecificLocalTime(&tzInfo, &s, &local)) {
				wsprintf((LPSTR) tbuf, (LPSTR) Format(4),
						local.wHour, local.wMinute, local.wSecond,
						local.wDay,
						(LPSTR) rstring(IDS_MONTH_NAMES + (local.wMonth - 1)), local.wYear);
			} else {
				strcpy(tbuf, "");
			}
		}
    } else {
		set_tm_time(&gm, TRUE);

        /* Local time */

        wsprintf((LPSTR) tbuf, (LPSTR) Format(4),
                gm.tm_hour, gm.tm_min, gm.tm_sec,
                gm.tm_mday,
                (LPSTR) rstring(IDS_MONTH_NAMES + gm.tm_mon), gm.tm_year + 1900);
    }
    prt(3);

    /* Information about the Moon */

    /* Age of moon */

    wsprintf((LPSTR) tbuf, (LPSTR) Format(5),
            Plural(aom_d, IDS_DAY), Plural(aom_h, IDS_HOUR),
            Plural(aom_m, IDS_MINUTE));
    prt(5);

    /* Moon phase */

    wsprintf((LPSTR) tbuf, (LPSTR) Format(6),
            (int) (cphase * 100));
    prt(6);

    /* Moon distance */

    sprintf(tbuf, Format(7),
            (long) cdist, cdist / earthrad);
    prt(7);

    /* Moon subtends */

    sprintf(tbuf, Format(8), cangdia);
    prt(8);

    /* Information about the Sun */

    /* Sun's distance */

    sprintf(tbuf, Format(9),
            csund, csund / sunsmax);
    prt(10);

    /* Sun subtends */

    sprintf(tbuf, Format(10), csuang);
    prt(11);

    /* Calculate times of phases of this lunation.  This is
       sufficiently time-consuming that we only do it once a month. */

    if (repaint || (jd > nptime)) {
        phasehunt(jd + 0.5, phasar);
        lunation = (int) floor(((phasar[0] + 7) - lunatbase) / synmonth) + 1;

        for (i = 0; i < 5; i++) {
            fmt_phase_time(phasar[i], tbuf);
            prt(APOS(i));
        }
        nptime = phasar[4];

        /* Edit lunation numbers into cells reserved for them. */

        wsprintf((LPSTR) tbuf, (LPSTR) Format(11), lunation);
        prl(APOS(0));
        wsprintf((LPSTR) tbuf, (LPSTR) Format(11), lunation + 1);
        prl(APOS(4));
    }

    SelectObject(hDC, oBrush);
    DeleteObject(hBrush);
}
#undef APOS

/*  FMT_PHASE_TIME  --  Format  the  provided  julian  date  into  the
                        provided  buffer  in  UTC  format  for  screen
                        display  */

void fmt_phase_time(double utime, char *buf)
{
    int yy, mm, dd, hh, mmm, ss;

    jyear(utime, &yy, &mm, &dd);
    jhms(utime, &hh, &mmm, &ss);
    wsprintf((LPSTR) buf, (LPSTR) Format(12),
            hh, mmm, dd, (LPSTR) rstring(IDS_MONTH_NAMES + (mm - 1)), yy);
}

//  SET_TM_TIME  --  Set time from Windows system or local time

void set_tm_time(struct tm *t, BOOL islocal)
{
#define CtF(tf, sf) t->tf = s.sf

	SYSTEMTIME s;

	if (islocal) {
		GetLocalTime(&s);
	} else {
		GetSystemTime(&s);
	}
	CtF(tm_sec, wSecond);
	CtF(tm_min, wMinute);
	CtF(tm_hour, wHour);
	CtF(tm_mday, wDay);
	CtF(tm_mon, wMonth - 1);
	CtF(tm_year, wYear - 1900);
	CtF(tm_wday, wDayOfWeek);
	//  tm_yday  never used
	t->tm_isdst = GetTimeZoneInformation(&tzInfo) == TIME_ZONE_ID_DAYLIGHT;
#undef CtF
}

#ifdef NEEDED
//	SET_SYSTEM_TIME  --  Fill Windows system time from Unix tm struct

static void set_system_time(SYSTEMTIME *s, struct tm *t)
{
#define CtF(tf, sf) s->sf = t->tf

	CtF(tm_sec, wSecond);
	CtF(tm_min, wMinute);
	CtF(tm_hour, wHour);
	CtF(tm_mday, wDay);
	CtF(tm_mon + 1, wMonth);
	CtF(tm_year + 1900, wYear);
	CtF(tm_wday, wDayOfWeek);
	s->wMilliseconds = 0;
#undef CtF
}
#endif

/*  JTIME  --  Convert a Unix date and time (tm) structure to astronomical
			   Julian time (i.e. Julian date plus day fraction,
			   expressed as a double).  */

double jtime(struct tm *t)
{
    return ucttoj(t->tm_year + 1900, t->tm_mon, t->tm_mday, t->tm_hour, t->tm_min, t->tm_sec);
}

/*  UCTTOJ  --	Convert GMT date and time to astronomical
				Julian time (i.e. Julian date plus day fraction,
				expressed as a double).  */

double ucttoj(long year, int mon, int mday,
	      	  int hour, int min, int sec)
{

	// Algorithm as given in Meeus, Astronomical Algorithms, Chapter 7, page 61

	int a, b, m;
	long y;

#ifdef PARANOID
    assert(mon  >= 0 && mon  < 12);
    assert(mday >  0 && mday < 32);
    assert(hour >= 0 && hour < 24);
    assert(min  >= 0 && min  < 60);
    assert(sec  >= 0 && sec  < 60);
#endif

    m = mon + 1;
    y = year;

	if (m <= 2) {
		y--;
		m += 12;
	}

	/* Determine whether date is in Julian or Gregorian calendar based on
	   canonical date of calendar reform. */

	if ((year < 1582) || ((year == 1582) && ((mon < 9) || (mon == 9 && mday < 5)))) {
		b = 0;
	} else {
		a = ((int) (y / 100));
		b = 2 - a + (a / 4);
	}

	return (((long) (365.25 * (y + 4716))) + ((int) (30.6001 * (m + 1))) +
				mday + b - 1524.5) +
			((sec + 60L * (min + 60L * hour)) / 86400.0);
}

/*  JYEAR  --  Convert	Julian	date  to  year,  month, day, which are
			   returned via integer pointers to integers (note that year is a long).  */

void jyear(double td, long *yy, int *mm, int *dd)
{
	double z, f, a, alpha, b, c, d, e;

	td += 0.5;
	z = floor(td);
	f = td - z;

	if (z < 2299161.0) {
		a = z;
	} else {
		alpha = floor((z - 1867216.25) / 36524.25);
		a = z + 1 + alpha - floor(alpha / 4);
	}

	b = a + 1524;
	c = floor((b - 122.1) / 365.25);
	d = floor(365.25 * c);
	e = floor((b - d) / 30.6001);

	*dd = (int) (b - d - floor(30.6001 * e) + f);
	*mm = (int) ((e < 14) ? (e - 1) : (e - 13));
	*yy = (long) ((*mm > 2) ? (c - 4716) : (c - 4715));
}

/*  JHMS  --  Convert Julian time to hour, minutes, and seconds.  */

void jhms(double j, int *h, int *m, int *s)
{
    long ij;

    j += 0.5;			      /* Astronomical to civil */
    ij = (long) (((j - floor(j)) * 86400.0) + 0.5);  // Round to nearest second
    *h = (int) (ij / 3600L);
    *m = (int) ((ij / 60L) % 60L);
    *s = (int) (ij % 60L);
}

/*	JWDAY  --  Determine day of the week for a given Julian day.  */

int jwday(double j)
{
	return ((int) (j + 1.5)) % 7;
}

/*  MEANPHASE  --  Calculates  time  of  the mean new Moon for a given
                   base date.  This argument K to this function is the
                   precomputed synodic month index, given by:

                          K = (year - 1900) * 12.3685

                   where year is expressed as a year and fractional year.  */

static double meanphase(double sdate, double k)
{
    double t, t2, t3, nt1;

    /* Time in Julian centuries from 1900 January 0.5 */
    t = (sdate - 2415020.0) / 36525;
    t2 = t * t;                       /* Square for frequent use */
    t3 = t2 * t;                      /* Cube for frequent use */

    nt1 = 2415020.75933 + synmonth * k
            + 0.0001178 * t2
            - 0.000000155 * t3
            + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

    return nt1;
}

/*  TRUEPHASE  --  Given a K value used to determine the mean phase of
                   the new moon, and a phase selector (0.0, 0.25, 0.5,
                   0.75), obtain the true, corrected phase time.  */

static double truephase(double k, double phase)
{
    double t, t2, t3, pt, m, mprime, f;
    int apcor = FALSE;

    k += phase;                       /* Add phase to new moon time */
    t = k / 1236.85;                  /* Time in Julian centuries from
                                         1900 January 0.5 */
    t2 = t * t;                       /* Square for frequent use */
    t3 = t2 * t;                      /* Cube for frequent use */
    pt = 2415020.75933                /* Mean time of phase */
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
        MessageBox(hWndMain, rstring(IDS_ERR_TRUEPHASE),
        rstring(IDS_ERR_IERR), MB_ICONEXCLAMATION | MB_OK | MB_APPLMODAL);
        PostQuitMessage(1);
    }
    return pt;
}

/*   PHASEHUNT  --  Find time of phases of the moon which surround the
                    current date.  Five phases are found, starting and
                    ending with the new moons which bound the  current
                    lunation.  */

void phasehunt(double sdate, double phases[5])
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

/*  KEPLER  --   Solve the equation of Kepler.  */

static double kepler(double m, double ecc)
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
    stores into pointer arguments  the  illuminated  fraction  of  the
    Moon's  disc, the Moon's age in days and fraction, the distance of
    the Moon from the centre of the Earth, and  the  angular  diameter
    subtended  by the Moon as seen by an observer at the centre of the
    Earth.
*/

double phase(
  double  pdate,                      /* Date for which to calculate phase */
  double  *pphase,                    /* Illuminated fraction */
  double  *mage,                      /* Age of moon in days */
  double  *dist,                      /* Distance in kilometres */
  double  *angdia,                    /* Angular diameter in degrees */
  double  *sudist,                    /* Distance to Sun */
  double  *suangdia)                  /* Sun's angular diameter */
{

    double Day, N, M, Ec, Lambdasun, ml, MM, MN, Ev, Ae, A3, MmP,
           mEc, A4, lP, V, lPP, NP, y, x, Lambdamoon, BetaM,
           MoonAge, MoonPhase,
           MoonDist, MoonDFrac, MoonAng, MoonPar,
           F, SunDist, SunAng;

    /* Calculation of the Sun's position */

    Day = pdate - epoch;                    /* Date within epoch */
    N = fixangle((360 / 365.2422) * Day);   /* Mean anomaly of the Sun */
    M = fixangle(N + elonge - elongp);      /* Convert from perigee
                                               co-ordinates to epoch 1980.0 */
    Ec = kepler(M, eccent);                 /* Solve equation of Kepler */
    Ec = sqrt((1 + eccent) / (1 - eccent)) * tan(Ec / 2);
    Ec = 2 * todeg(atan(Ec));               /* True anomaly */
    Lambdasun = fixangle(Ec + elongp);      /* Sun's geocentric ecliptic
                                               longitude */
    /* Orbital distance factor */
    F = ((1 + eccent * cos(torad(Ec))) / (1 - eccent * eccent));
    SunDist = sunsmax / F;                  /* Distance to Sun in km */
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
