/*

                          Moontool for Windows
                               Release 2.0

    Designed and implemented by John Walker in December 1987.
    Revised and updated in July of 1989 by Ron Hitchens.
    Converted to Microsoft Windows in January of 1992 by John Walker.
	Convert to Win32 in March of 1999 by John Walker.

    The  algorithms used in this program to calculate the positions of
    the Sun and Moon as seen from the Earth  are  given  in  the  book
    "Practical Astronomy With Your Calculator" by Peter Duffett-Smith,
    Second Edition, Cambridge University Press, 1981.  Ignore the word
    "Calculator"  in  the  title;  this  is  an essential reference if
    you're  interested  in  developing   software   which   calculates
    planetary  positions,  orbits,  eclipses, and the like.  If you're
    interested in pursuing such programming, you should  also  obtain:

    "Astronomical Formulae  for  Calculators"  by  Jean  Meeus,  Third
    Edition, Willmann-Bell, 1985.  A must-have.

    "Planetary  Programs  and  Tables  from  -4000 to +2800" by Pierre
    Bretagnon and Jean-Louis Simon, Willmann-Bell, 1986.  If you  want
    the  utmost  (outside of JPL) accuracy for the planets, it's here.

    "Celestial BASIC" by Eric Burgess, Revised Edition,  Sybex,  1985.
    Very cookbook oriented, and many of the algorithms are hard to dig
    out of the turgid BASIC code, but you'll probably want it anyway.

    Many  of these references can be obtained from Willmann-Bell, P.O.
    Box 35025, Richmond, VA 23235, USA.  Phone:  (804)  320-7016.   In
    addition  to  their  own  publications,  they  stock  most  of the
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
        June 1988       Version 2.0 for the Sun workstation posted
                        to usenet by John Walker

        June 1988       Modified by Ron Hitchens

        July 1989       Modified a little bit more to use an accurate
                        grey-scale moon face created by Joe Hitchens
                        on an Amiga.
                        Added The Apollo 11 Commemorative Red Dot, to show
                        where Neil and Buzz went on vacation 20 years ago.

        March 1992      Moontool for Windows 1.0 implemented by John Walker.

        April 1992      Bug fix update 1.01 correcting problems reported by
                        Michael Geary.

		March 1999		Win32 version, reverting to standard Julian Day
						definition after ill-conceived flirtation with "civil
						Julian Day" beginning at midnight.  Release 2.0.

*/

#include "moontool.h"

struct moongeom mgeom;              /* Geometry definitions */
char szString[128];					/* Variable to load resource strings */
char szAppName[20];					/* Class name for the window */
HINSTANCE hInst;					/* Class instance pointer */
HWND hWndMain;						/* Main window pointer */
HICON moontoolIcon = NULL;			/* Application icon */
NOTIFYICONDATA trayIcon;			// Taskbar tray icon
BOOL inTray = FALSE;				// Shown in system tray ?
static int holped = FALSE;			// Did we invoke help ?
static HMENU popup = NULL;			// Popup menu for tray

#define Timer ((WORD) (testmode ? 250 : (IsIconic(hWnd) ? 32000 : 1000)))

/*  UPDATE_WINDOW  --  Update the contents of the window.  If
                       REPAINT is nonzero, the entire contents
                       of the window is updated.  Otherwise,
                       only changed material is displayed. */

static void update_window(HWND hWnd, HDC hDC, int repaint)
{

    SetTextColor(hDC, GetSysColor(COLOR_WINDOWTEXT));
    SetBkColor(hDC, GetSysColor(COLOR_WINDOW));

    if (repaint && !IsIconic(hWnd)) {
        SetWindowText(hWndMain, rstring(IDS_APPNAME));
        paint_labels(hDC);
    }
    ringgg(hWnd, hDC, repaint);
}

/*

    nCwRegisterClasses Function

    The following function  registers  all  the  classes  of  all  the
    windows associated with this application.  The function returns an
    error code if unsuccessful, otherwise it returns 0.

*/

static int nCwRegisterClasses(void)
{
    WNDCLASS wndclass;          /* struct to define a window class */

    memset(&wndclass, 0x00, sizeof(WNDCLASS));

    /* load WNDCLASS with window's characteristics */
    wndclass.style = CS_HREDRAW | CS_VREDRAW | CS_BYTEALIGNWINDOW;
    wndclass.lpfnWndProc = WndProc;
    /* Extra storage for Class and Window objects */
    wndclass.cbClsExtra = 0;
    wndclass.cbWndExtra = 0;
    wndclass.hInstance = hInst;
    wndclass.hIcon = moontoolIcon = LoadIcon(hInst, MAKEINTRESOURCE(MOONTOOL_ICON));
    wndclass.hCursor = LoadCursor(NULL, IDC_ARROW);
    /* Create brush for erasing background */
    wndclass.hbrBackground = (HBRUSH) (COLOR_WINDOW + 1);
    wndclass.lpszMenuName = szAppName;      /* Menu Name is App Name */
    wndclass.lpszClassName = szAppName;     /* Class Name is App Name */
    if (!RegisterClass(&wndclass)) {
        return -1;
	}
    return 0;
}

/*

    Windows Main Program

*/

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpszCmdLine, int nCmdShow)
{
    MSG msg;						/* MSG structure to store your messages */
    int nRc;						/* return value from Register Classes */
    static HANDLE hlr;
    struct moongeom far *geometry;

    strcpy(szAppName, "MOONTOOL");
    hInst = hInstance;
    if (!hPrevInstance) {
        /* Register window classes if first instance of application */
        if ((nRc = nCwRegisterClasses()) == -1) {
            /* Registering one of the windows failed */
            LoadString(hInst, IDS_ERR_REGISTER_CLASS, szString, sizeof(szString));
            MessageBox(NULL, szString, NULL, MB_ICONEXCLAMATION);
            return nRc;
        }
    }

    hlr = LoadResource(hInst, FindResource(hInst, (LPSTR) "Geometry",
            RT_RCDATA));
    if (hlr == NULL) {
        MessageBox(NULL, "Bogus!", NULL, MB_ICONEXCLAMATION);
        return -1;
    }
    geometry = (struct moongeom far *) LockResource(hlr);
    mgeom = *geometry;
    FreeResource(hlr);

    /* Create application's Main window */

    hWndMain = CreateWindow(
        szAppName,              // Window class name
        rstring(IDS_APPNAME),   // Window's title
        WS_CAPTION      |       // Title and Min/Max
        WS_SYSMENU      |       // Add system menu box
        WS_MINIMIZEBOX  |       // Add minimize box
        WS_BORDER       |       // thin frame
        WS_CLIPCHILDREN |       // don't draw in child windows areas
        WS_OVERLAPPED,          // this is a conventional overlapped window
        CW_USEDEFAULT, 0,       // Use default X, Y
        CW_USEDEFAULT, 0,       // Use default X, Y
        NULL,                   // Parent window's handle
        NULL,                   // Default to Class Menu
        hInst,                  // Instance of window
        NULL);                  // Create struct for WM_CREATE

    if (hWndMain == NULL) {
        LoadString(hInst, IDS_ERR_CREATE_WINDOW, szString, sizeof(szString));
        MessageBox(NULL, szString, NULL, MB_ICONEXCLAMATION);
        return IDS_ERR_CREATE_WINDOW;
    }

    ShowWindow(hWndMain, nCmdShow);     /* Display main window */

    while (GetMessage(&msg, NULL, 0, 0)) {
        /* Until WM_QUIT message */
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    /* Do clean up before exiting from the application */

    return msg.wParam;
}

/*
    Main Window Procedure

*/

#define WM_TRAY_CLICK	(WM_USER + 100)			//	Click in tray notification

LRESULT CALLBACK WndProc(HWND hWnd, UINT Message, WPARAM wParam, LPARAM lParam)
{
    HMENU hMenu = NULL;					// Handle for the menu
    HBITMAP hBitmap = NULL;				// Handle for bitmaps
    HDC hDC;							// Handle for the display context
    PAINTSTRUCT ps;						// Window paint information
    int nRc = 0;						// Utility status from function calls
    TEXTMETRIC tm;						// System font text metric
    static short cxClient, cyClient;    // Window size
    static int wasIconic = -2;			// Iconic status

    switch (Message) {

        case WM_COMMAND:

         switch (wParam) {

            case IDM_F_EXIT:
			case ID_POPUP_EXIT:
                SendMessage(hWnd, WM_CLOSE, 0, 0);
                break;

            case IDM_F_COPY:
                {
                    HDC hdcMem;
                    HBITMAP hBitmap;

                    hDC = GetDC(hWnd);
                    hdcMem = CreateCompatibleDC(hDC);
                    hBitmap = CreateCompatibleBitmap(hDC,
                                cxClient, cyClient);
                    if (hBitmap) {
                        SelectObject(hdcMem, hBitmap);
                        BitBlt(hdcMem, 0, 0, cxClient, cyClient,
                                hDC, 0, 0, SRCCOPY);
                        OpenClipboard(hWnd);
                        EmptyClipboard();
                        SetClipboardData(CF_BITMAP, hBitmap);
                        CloseClipboard();
                        DeleteDC(hdcMem);
                        ReleaseDC(hWnd, hDC);
                    }
                }
                break;

#ifdef _DEBUG
			
			/*	Copy entire window including title, menu, etc.  This
				is compiled in only in _DEBUG builds and is intended
				to permit making screen shots on brain-dead NT where
				grab screen is broken.  */

#define		IDM_F_COPY_WINDOW	(IDM_F_COPY + 1)

            case IDM_F_COPY_WINDOW:
                {
					HWND desktop = GetDesktopWindow();
                    HDC hdcMem;
                    HBITMAP hBitmap;
					RECT extents;

                    hDC = GetDC(desktop);
					GetWindowRect(hWnd, &extents);
                    hdcMem = CreateCompatibleDC(hDC);
                    hBitmap = CreateCompatibleBitmap(hDC,
                                extents.right - extents.left, extents.bottom - extents.top);
                    if (hBitmap) {
                        SelectObject(hdcMem, hBitmap);
                        BitBlt(hdcMem, 0, 0, extents.right - extents.left, extents.bottom - extents.top,
                                hDC, extents.left, extents.top, SRCCOPY);
                        OpenClipboard(hWnd);
                        EmptyClipboard();
                        SetClipboardData(CF_BITMAP, hBitmap);
                        CloseClipboard();
                        DeleteDC(hdcMem);
                        ReleaseDC(desktop, hDC);
                    }
                }
                break;
#endif

             case IDM_F_TESTMODE:
                KillTimer(hWnd, 1);
                testmode = !testmode;
                SetTimer(hWnd, 1, Timer, NULL);
                hDC = GetDC(hWnd);
                update_window(hWnd, hDC, 1);
                ReleaseDC(hWnd, hDC);
                break;

             case IDM_F_STOP:
                runmode = FALSE;
                ModifyMenu(GetMenu(hWnd), IDM_F_STOP, MF_BYCOMMAND,
                                  IDM_F_RUN, (LPSTR) "&Run");
                hDC = GetDC(hWnd);
                update_window(hWnd, hDC, 1);
                ReleaseDC(hWnd, hDC);
                break;

             case IDM_F_RUN:
                runmode = TRUE;
                ModifyMenu(GetMenu(hWnd), IDM_F_RUN, MF_BYCOMMAND,
                                  IDM_F_STOP, (LPSTR) "&Stop");
                hDC = GetDC(hWnd);
                update_window(hWnd, hDC, 1);
                ReleaseDC(hWnd, hDC);
                break;

             case IDM_F_ABOUT:
				DialogBox(hInst, MAKEINTRESOURCE(IDD_ABOUT), hWnd, About);
                break;

             case IDM_F_TIMEZONE:
				DialogBox(hInst, MAKEINTRESOURCE(IDD_TIMEZONE), hWnd, About);
                break;

             case IDM_F_SETJDATE:
                  nRc = DialogBox(hInst, MAKEINTRESOURCE(IDD_SET_JULIAN), hWnd, SetJdate);
                  if (nRc) {
                    runmode = FALSE;
                    ModifyMenu(GetMenu(hWnd), IDM_F_STOP, MF_BYCOMMAND,
                              IDM_F_RUN, (LPSTR) "&Run");
                    hDC = GetDC(hWnd);
                    update_window(hWnd, hDC, 1);
                    ReleaseDC(hWnd, hDC);
                  }
                break;

             case IDM_F_SETUTIME:
				nRc = DialogBox(hInst, MAKEINTRESOURCE(IDD_SET_UTC), hWnd, SetUtime);
				if (nRc) {
					runmode = FALSE;
					ModifyMenu(GetMenu(hWnd), IDM_F_STOP, MF_BYCOMMAND,
							   IDM_F_RUN, (LPSTR) "&Run");
					hDC = GetDC(hWnd);
					update_window(hWnd, hDC, 1);
					ReleaseDC(hWnd, hDC);
				}
                break;

             case IDM_HELP_CONTENTS:
             	WinHelp(hWnd, rstring(IDS_HELPFILE), HELP_CONTENTS, 0L);
             	holped = TRUE;
             	break;

             case IDM_HELP_SEARCH:
             	WinHelp(hWnd, rstring(IDS_HELPFILE), HELP_PARTIALKEY, ((DWORD) ((LPSTR) "")));
             	holped = TRUE;
             	break;

			case ID_POPUP_OPEN:
				ShowWindow(hWnd, SW_SHOW);
				ShowWindow(hWnd, SW_RESTORE);
				break;

            default:
                return DefWindowProc(hWnd, Message, wParam, lParam);
            }
            break;

        case WM_CREATE:
            hDC = GetDC(hWnd);
            GetTextMetrics(hDC, &tm);
            SetWindowPos(hWnd, 0, 0, 0,
                tm.tmMaxCharWidth * mgeom.winwid,
                (tm.tmHeight + tm.tmExternalLeading) * mgeom.winhgt,
                SWP_NOMOVE + SWP_NOZORDER);
            ReleaseDC(hWnd, hDC);
			GetTimeZoneInformation(&tzInfo);
#ifdef _DEBUG
			AppendMenu(GetSubMenu(GetMenu(hWnd), 1), MF_STRING, IDM_F_COPY_WINDOW, "Copy &Window");
#endif
            break;

        case WM_DESTROY:
            KillTimer(hWnd, 1);
			if (inTray) {
 				Shell_NotifyIcon(NIM_DELETE, &trayIcon);
			}
			if (popup != NULL) {
				DestroyMenu(popup);
			}
			if (moontoolIcon != NULL) {
				DestroyIcon(moontoolIcon);
			}
            if (holped) {
            	WinHelp(hWnd, rstring(IDS_HELPFILE), HELP_QUIT, 0L);
            }
            PostQuitMessage(0);
            break;

		case WM_KEYDOWN:
			{
				int vkey = (int) wParam;

				if (vkey == VK_F1) {
					PostMessage(hWnd, WM_COMMAND, IDM_HELP_CONTENTS, 0);
				}
			}
			break;

        case WM_SIZE:
            /* wParam contains a code indicating the requested sizing */
            cxClient = LOWORD(lParam);
            cyClient = HIWORD(lParam);
            /* lParam contains the new height and width of the client area */
			if (wParam == SIZE_MINIMIZED) {
				PostMessage(hWnd, WM_PAINT, 0, 0);
			}
            break;

		/*	If we're iconic, restore the normal window if the user
			double clicks on our tray icon.  */

		case WM_TRAY_CLICK:
			if ((lParam == WM_LBUTTONDBLCLK) && IsIconic(hWnd)) {
				ShowWindow(hWnd, SW_SHOW);
				ShowWindow(hWnd, SW_RESTORE);
			} else if ((lParam == WM_RBUTTONDOWN) && IsIconic(hWnd)) {
				POINT here;

				if (popup == NULL) {
					popup = LoadMenu(hInst, MAKEINTRESOURCE(IDR_POPUP));
				}
				GetCursorPos(&here);
				TrackPopupMenu(GetSubMenu(popup, 0), TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
							   here.x, here.y, 0, hWnd, NULL);
			}
			break;

        case WM_PAINT:
            memset(&ps, 0, sizeof(PAINTSTRUCT));
            hDC = BeginPaint(hWnd, &ps);

            /* If iconic state of the window has  changed,  reset  the
               timer  to the update rate appropriate to the new window
               state.  */

            if (IsIconic(hWnd) != wasIconic) {
                if (wasIconic != -2) {
                    KillTimer(hWnd, 1);
                }
                wasIconic = IsIconic(hWnd);
                SetTimer(hWnd, 1, Timer, NULL);
				/*	Since we already have an icon in the system tray,
					hide the usual taskbar icon when minimised.  */
                if (wasIconic) {
                    go_iconic();
					trayIcon.cbSize = sizeof(NOTIFYICONDATA);
					trayIcon.hWnd = hWnd;
					trayIcon.uID = 1;
					trayIcon.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
					trayIcon.uCallbackMessage = WM_TRAY_CLICK;
					trayIcon.hIcon = moontoolIcon;
					strcpy(trayIcon.szTip, rstring(IDS_APPNAME));
					inTray = Shell_NotifyIcon(NIM_ADD, &trayIcon) != 0;
					ShowWindow(hWnd, SW_HIDE);
                } else {
					ShowWindow(hWnd, SW_SHOW);
					if (inTray) {
						Shell_NotifyIcon(NIM_DELETE, &trayIcon);
						inTray = FALSE;
					}
				}
            }

            /* Included in case the background is not a pure color  */
            SetBkMode(hDC, TRANSPARENT);

            /* Application should draw on the client window using  the
               GDI  graphics and text functions.  'ps' the PAINTSTRUCT
               returned by BeginPaint contains a rectangle to the area
               that must be repainted.  */

            if (wasIconic) {
                /* Clear the icon frame to black */
                BitBlt(hDC, 0,0, 9999, 9999, NULL, 0, 0, BLACKNESS);
            }
            update_window(hWnd, hDC, 1);
            /* Inform Windows painting is complete */
            EndPaint(hWnd, &ps);
            break;

        case WM_SYSCOLORCHANGE:
            /* Since the system window background and text colour settings
               are used for the open window display, force the window to
               be repainted when the system colours change. */
            InvalidateRect(hWnd, NULL, TRUE);
            break;

        case WM_TIMER:
            hDC = GetDC(hWnd);
            update_window(hWnd, hDC, 0);
            ReleaseDC(hWnd, hDC);
            break;

        case WM_CLOSE:
            /* Destroy child windows, modeless dialogs, then this window.  */
            DestroyWindow(hWnd);
            if (hWnd == hWndMain)
            PostQuitMessage(0); /* Quit the application */
            break;

        case WM_INITMENU:
            if (GetMenu(hWnd) == ((HMENU) wParam)) {
                CheckMenuItem((HMENU) wParam, IDM_F_TESTMODE,
                    testmode ? MF_CHECKED : MF_UNCHECKED);
            }
            break;

        default:
            /* For  any  message  for  which  you  don't  specifically
               provide  a  service  routine,  you  should  return  the
               message  to Windows for default message processing.  */
            return DefWindowProc(hWnd, Message, wParam, lParam);
    }
    return 0L;
}
