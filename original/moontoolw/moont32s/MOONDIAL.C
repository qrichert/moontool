/*

            Moontool for Windows

    Dialogue message processing functions

*/

#include "moontool.h"

/*  ABOUT  --  Dialogue message procedure for the About dialogue. */

BOOL CALLBACK About(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam)
{
    switch (message) {
        case WM_INITDIALOG:
            return TRUE;

        case WM_COMMAND:
            EndDialog(hDlg, TRUE);
            return TRUE;
    }
    return FALSE;
}

/*  SETJDATE  --  Dialogue message procedure for the Set Julian Date
                  dialogue. */

BOOL CALLBACK SetJdate(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam)
{
    int yy, mm, dd, hh, mmm, ss;
    static double jd;
    struct tm gm;
    char tbuf[80];

    switch (message) {
        case WM_INITDIALOG:
            jd = faketime;
            sprintf(tbuf, Format(2), jd);
            SetDlgItemText(hDlg, IDC_J_JDATE, (LPSTR) tbuf);
            jyear(jd, &yy, &mm, &dd);
            jhms(jd, &hh, &mmm, &ss);
            sprintf(tbuf, Format(3),
                hh, mmm, ss, dd,
                rstring(IDS_MONTH_NAMES + (mm - 1)), yy);
            SetDlgItemText(hDlg, IDC_J_UTIME, (LPSTR) tbuf);
            return TRUE;

        case WM_COMMAND:
            switch (LOWORD(wParam)) {
                case IDOK:
                    EndDialog(hDlg, TRUE);
                    faketime = jd;
                    return TRUE;

                case IDCANCEL:
                    EndDialog(hDlg, FALSE);
                    return TRUE;

                case IDC_J_NOW:
					set_tm_time(&gm, FALSE);
                    jd = jtime(&gm);
                    sprintf(tbuf, Format(2), jd);
                    SetDlgItemText(hDlg, IDC_J_JDATE, (LPSTR) tbuf);
                    return TRUE;

                case IDC_J_JDATE:
                    if (HIWORD(wParam) == EN_CHANGE) {
                        GetDlgItemText(hDlg, IDC_J_JDATE,
                            (LPSTR) tbuf, (sizeof tbuf) - 1);
                        if (sscanf(tbuf, "%lf", &jd) > 0 && (jd >= 0)) {
//                          jd -= 0.5;
                            jyear(jd, &yy, &mm, &dd);
                            jhms(jd, &hh, &mmm, &ss);
                            sprintf(tbuf, Format(3),
                                hh, mmm, ss, dd,
                                rstring(IDS_MONTH_NAMES + (mm - 1)), yy);
                            SetDlgItemText(hDlg, IDC_J_UTIME, (LPSTR) tbuf);
                        } else {
                            SetDlgItemText(hDlg, IDC_J_UTIME, (LPSTR) Format(14));
                        }
                    }
                    return TRUE;

                default:
                    break;
            }
            break;
    }
    return FALSE;
}

/*  SET_UTIME_FROM_JD  --  Decompose a Julian date into UTC date and
                           time and fill the edit fields with the
                           components. */

static void set_utime_from_jd(HWND hDlg, double jd)
{
    int yy, mm, dd, hh, mmm, ss;
    char tbuf[80];

    jyear(jd, &yy, &mm, &dd);
    jhms(jd, &hh, &mmm, &ss);
    sprintf(tbuf, "%d", yy);
    SetDlgItemText(hDlg, IDC_U_YEAR, (LPSTR) tbuf);
    sprintf(tbuf, "%d", dd);
    SetDlgItemText(hDlg, IDC_U_DAY, (LPSTR) tbuf);
    sprintf(tbuf, "%d", hh);
    SetDlgItemText(hDlg, IDC_U_HOUR, (LPSTR) tbuf);
    sprintf(tbuf, "%d", mmm);
    SetDlgItemText(hDlg, IDC_U_MINUTE, (LPSTR) tbuf);
    sprintf(tbuf, "%d", ss);
    SetDlgItemText(hDlg, IDC_U_SECOND, (LPSTR) tbuf);
    SendDlgItemMessage(hDlg, IDC_U_MONTH, CB_SETCURSEL, mm - 1, 0L);
}

/*  CHANGE_UTC  --  Change the Julian date when one or more
                    components of the UTC change. */

static void change_utc(HWND hDlg, double *jd, int nyy, int nmm, int ndd,
                                              int nhh, int nmmm, int nss)
{
    int yy, mm, dd, hh, mmm, ss;
    char tbuf[80];

    jyear(*jd, &yy, &mm, &dd);
    jhms(*jd, &hh, &mmm, &ss);

    mm--;
#define Update(x) if (n ## x != -1) { x = n ## x; }
    Update(yy);
    Update(mm);
    Update(dd);
    Update(hh);
    Update(mmm);
    Update(ss);
#undef Update

    *jd = ucttoj(yy, mm, dd, hh, mmm, ss);
    sprintf(tbuf, Format(2), *jd);
    SetDlgItemText(hDlg, IDC_U_JDATE, (LPSTR) tbuf);
}

/*  SETUTIME  --  Dialogue message procedure for the Set Universal Time
                  dialogue.  */

BOOL CALLBACK SetUtime(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam)
{
    int i;
    static double jd;
    struct tm gm;
    BOOL ok;
    char tbuf[80];

    switch (message) {
        case WM_INITDIALOG:

            jd = faketime;
            SendDlgItemMessage(hDlg, IDC_U_MONTH, WM_SETREDRAW, FALSE, 0L);
            for (i = 0; i < 12; i++) {
                SendDlgItemMessage(hDlg, IDC_U_MONTH, CB_ADDSTRING,
                    0,
                    (LPARAM) (LPCSTR) rstring(IDS_MONTH_NAMES + i));
            }
            SendDlgItemMessage(hDlg, IDC_U_MONTH, WM_SETREDRAW, TRUE, 0L);
            sprintf(tbuf, Format(2), jd);
            SetDlgItemText(hDlg, IDC_U_JDATE, (LPSTR) tbuf);
            set_utime_from_jd(hDlg, jd);
            return TRUE;

        case WM_COMMAND:
            switch (LOWORD(wParam)) {
                case IDOK:
                    EndDialog(hDlg, TRUE);
                    faketime = jd;
                    return TRUE;

                case IDCANCEL:
                    EndDialog(hDlg, FALSE);
                    return TRUE;

                case IDC_U_NOW:
					set_tm_time(&gm, FALSE);
                    jd = jtime(&gm);
                    set_utime_from_jd(hDlg, jd);
                    return TRUE;

                case IDC_U_YEAR:
                    if (HIWORD(wParam) == EN_CHANGE) {
                        int y = (int) GetDlgItemInt(hDlg, IDC_U_YEAR,
                            &ok, TRUE);
                        if (ok) {
                            change_utc(hDlg, &jd, y, -1, -1, -1, -1, -1);
                        }
                    }
                    return TRUE;

                case IDC_U_MONTH:
                    if (HIWORD(wParam) == CBN_SELCHANGE) {
                        WORD v = (WORD) SendDlgItemMessage(hDlg, IDC_U_MONTH,
                            CB_GETCURSEL, 0, 0L);
                        if (v != CB_ERR) {
                            change_utc(hDlg, &jd, -1, v, -1, -1, -1, -1);
                        }
                    }
                    return TRUE;

                case IDC_U_DAY:
                    if (HIWORD(wParam) == EN_CHANGE) {
                        WORD v = GetDlgItemInt(hDlg, IDC_U_DAY,
                            &ok, FALSE);
                        if (ok) {
                            change_utc(hDlg, &jd, -1, -1, v, -1, -1, -1);
                        }
                    }
                    return TRUE;

                case IDC_U_HOUR:
                    if (HIWORD(wParam) == EN_CHANGE) {
                        WORD v = GetDlgItemInt(hDlg, IDC_U_HOUR,
                            &ok, FALSE);
                        if (ok) {
                            change_utc(hDlg, &jd, -1, -1, -1, v, -1, -1);
                        }
                    }
                    return TRUE;

                case IDC_U_MINUTE:
                    if (HIWORD(wParam) == EN_CHANGE) {
                        WORD v = GetDlgItemInt(hDlg, IDC_U_MINUTE,
                            &ok, FALSE);
                        if (ok) {
                            change_utc(hDlg, &jd, -1, -1, -1, -1, v, -1);
                        }
                    }
                    return TRUE;

                case IDC_U_SECOND:
                    if (HIWORD(wParam) == EN_CHANGE) {
                        WORD v = GetDlgItemInt(hDlg, IDC_U_SECOND,
                            &ok, FALSE);
                        if (ok) {
                            change_utc(hDlg, &jd, -1, -1, -1, -1, -1, v);
                        }
                    }
                    return TRUE;

                default:
                    break;
            }
            break;
    }
    return FALSE;
}