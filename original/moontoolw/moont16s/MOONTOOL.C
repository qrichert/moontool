/*

                          Moontool for Windows
                             Release 1.02

    Designed and implemented by John Walker in December 1987.
    Revised and updated in July of 1989 by Ron Hitchens.
    Converted to Microsoft Windows in January of 1992 by John Walker.

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
                             ronbo@vixen.uucp
                             ...!uunet!cs.utah.edu!caeco!vixen!ronbo
                             hitchens@cs.utexas.edu

        July 1989       Modified a little bit more to use an accurate
                        grey-scale moon face created by Joe Hitchens
                        on an Amiga.
                        Added The Apollo 11 Commemorative Red Dot, to show
                        where Neil and Buzz went on vacation 20 years ago.

        March 1992      Moontool for Windows 1.0 implemented by John Walker.

        April 1992      Bug fix update 1.01 correcting problems reported by
                        Michael Geary.

        March 1999      Bug fix update 1.02 Minor Y2K fix, switch from
                        "civil Julian days" starting at midnight to
                        conventional Julian days which start at noon.

*/

#include "moontool.h"

struct moongeom mgeom;              /* Geometry definitions */
char szString[128];   /* Variable to load resource strings */
char szAppName[20];   /* Class name for the window */
HWND hInst;           /* Class instance pointer */
HWND hWndMain;        /* Main window pointer */


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

    Windows 3.0 Main Program Body

    The  following  routine  is  the  Windows  Main Program.  The Main
    Program is executed when a program is selected  from  the  Windows
    Control  Panel or File Manager.  The WinMain routine registers and
    creates the program's main window and initializes global  objects.
    The   WinMain  routine  also  includes  the  applications  message
    dispatch loop.  Every window message destined for the main  window
    or  any  subordinate windows is obtained, possibly translated, and
    dispatched  to  a  window  or  dialog  processing  function.   The
    dispatch  loop  is  exited  when  a  WM_QUIT  message is obtained.
    Before exiting the WinMain  routine  should  destroy  any  objects
    created and free memory and other resources.

*/

int PASCAL WinMain(HANDLE hInstance, HANDLE hPrevInstance, LPSTR lpszCmdLine, int nCmdShow)
{
 /*
    HANDLE hInstance;       handle for this instance
    HANDLE hPrevInstance;   handle for possible previous instances
    LPSTR  lpszCmdLine;     long pointer to exec command line
    int    nCmdShow;        Show code for main window display
 */

    MSG msg;       /* MSG structure to store your messages */
    int nRc;       /* return value from Register Classes */
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

    tzset();                /* Define time zone from TZ variable */

    hlr = LoadResource(hInst, FindResource(hInst, (LPSTR) "Geometry",
            RT_RCDATA));
    if (hlr == NULL) {
        MessageBox(NULL, "Bogus!", NULL, MB_ICONEXCLAMATION);
        return -1;
    }
    geometry = (struct moongeom far *) LockResource(hlr);
    mgeom = *geometry;
    UnlockResource(hlr);

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

    if (getenv("TZ") == NULL) {
        MessageBox(hWndMain, rstring(IDS_ERR_NOTZSET),
            rstring(IDS_ERR_TZWARN), MB_ICONQUESTION | MB_OK);
    }

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

    This  procedure  provides  service routines for the Windows events
    (messages) that Windows sends to the window, as well as  the  user
    initiated  events  (messages)  that  are  generated  when the user
    selects  the  action  bar  and  pulldown  menu  controls  or   the
    corresponding keyboard accelerators.

    The  SWITCH  statement shown below distributes the window messages
    to the respective message service routines, which are set apart by
    the  CASE  statements.   The  window  procedures  must  provide an
    appropriate service routine for its end user  initiated  messages,
    as  well  as the general Windows messages (ie.  WM_CLOSE message).
    If a message is sent to this  procedure  for  which  there  is  no
    programmed  CASE clause (i.e., no service routine), the message is
    defaulted to the DefWindowProc function, where it  is  handled  by
    Windows.

    For  the  end-user initiated messages, this procedure is concerned
    principally with the WM_COMMAND message.  The menu control ID  (or
    the   corresponding   accelerator  ID)  is  communicated  to  this
    procedure in the first message  parameter  (wParam).   The  window
    procedure  provides  a  major  CASE  statement  for the WM_COMMAND
    message and a subordinate SWITCH statement to provide CASE clauses
    for  the  message  service  routines  for the various menu item's,
    identified by their ID values.

    The message service routines for the individual menu items are the
    main  work  points in the program.  These service routines contain
    the units of work performed when the end user select  one  of  the
    menu  controls.   The  required  application  response  to  a menu
    control is programmed in its associated CASE clause.  The  service
    routines  may  contain subroutine calls to separately compiled and
    libraried routines, in-line calls to subroutines to be embodied in
    this source code module, or program statements entered directly in
    the CASE clauses.  Program control is switched to the  appropriate
    service  routine  when  Windows  recognizes the end user event and
    sends a WM_COMMAND message to the window procedure.   The  service
    routine  provides the appropriate application-specific response to
    the end user initiated event, then breaks to return control to the
    WinMain()  routine which continues to service the message queue of
    the window(s).

*/

LONG FAR PASCAL WndProc(HWND hWnd, WORD Message, WORD wParam, LONG lParam)
{
    HMENU hMenu=0;          // handle for the menu
    HBITMAP hBitmap=0;      // handle for bitmaps
    HDC hDC;                // handle for the display device
    PAINTSTRUCT ps;         // holds PAINT information
    int nRc=0;              // return code
    TEXTMETRIC tm;          // System font text metric
    static short cxClient, cyClient;    // Window size
    static int wasIconic = -2;

    switch (Message) {

        case WM_COMMAND:
         /* The Windows messages for  action  bar  and  pulldown  menu
            items are processed here.

            The  WM_COMMAND  message  contains  the  message ID in its
            first parameter (wParam).  This routine is  programmed  to
            SWITCH  on  the #define values generated by CASE:W for the
            menu items in the application's header (*.H) file.  The ID
            values   have   the  format,  IDM_itemname.   The  service
            routines for  the  various  menu  items  follow  the  CASE
            statements up to the generated BREAK statements.  */

         switch (wParam) {

            case IDM_F_EXIT:
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
                {
                      FARPROC lpfnMsgProc;

                      lpfnMsgProc = MakeProcInstance((FARPROC) About, hInst);
                      nRc = DialogBox(hInst, MAKEINTRESOURCE(IDM_ABOUT), hWnd, lpfnMsgProc);
                      FreeProcInstance(lpfnMsgProc);
                }

                break;

             case IDM_F_TIMEZONE:
                {
                      FARPROC lpfnMsgProc;

                      lpfnMsgProc = MakeProcInstance((FARPROC) About, hInst);
                      nRc = DialogBox(hInst, MAKEINTRESOURCE(IDM_TIMEZONE), hWnd, lpfnMsgProc);
                      FreeProcInstance(lpfnMsgProc);
                }

                break;

             case IDM_F_SETJDATE:
                {
                      FARPROC lpfnMsgProc;

                      lpfnMsgProc = MakeProcInstance((FARPROC) SetJdate, hInst);
                      nRc = DialogBox(hInst, MAKEINTRESOURCE(9000), hWnd, lpfnMsgProc);
                      FreeProcInstance(lpfnMsgProc);
                      if (nRc) {
                        runmode = FALSE;
                        ModifyMenu(GetMenu(hWnd), IDM_F_STOP, MF_BYCOMMAND,
                                  IDM_F_RUN, (LPSTR) "&Run");
                        hDC = GetDC(hWnd);
                        update_window(hWnd, hDC, 1);
                        ReleaseDC(hWnd, hDC);
                      }
                }
                break;

             case IDM_F_SETUTIME:
                {
                      FARPROC lpfnMsgProc;

                      lpfnMsgProc = MakeProcInstance((FARPROC) SetUtime, hInst);
                      nRc = DialogBox(hInst, MAKEINTRESOURCE(6000), hWnd, lpfnMsgProc);
                      FreeProcInstance(lpfnMsgProc);
                      if (nRc) {
                        runmode = FALSE;
                        ModifyMenu(GetMenu(hWnd), IDM_F_STOP, MF_BYCOMMAND,
                                  IDM_F_RUN, (LPSTR) "&Run");
                        hDC = GetDC(hWnd);
                        update_window(hWnd, hDC, 1);
                        ReleaseDC(hWnd, hDC);
                      }
                }
                break;

            default:
                return DefWindowProc(hWnd, Message, wParam, lParam);
            }
            break;

        case WM_CREATE:
         /* The  WM_CREATE  message is sent  once to a window when the
            window is created.   The  window  procedure  for  the  new
            window  receives this message after the window is created,
            but before the window becomes visible.

            Parameters:

               lParam  -  Points to a CREATESTRUCT structure with
                  the following form:

               typedef struct {
                  LPSTR     lpCreateParams;
                  HANDLE    hInst;
                  HANDLE    hMenu;
                  HWND      hwndParent;
                  int       cy;
                  int       cx;
                  int       y;
                  int       x;
                  LONG      style;
                  LPSTR     lpszName;
                  LPSTR     lpszClass;
                  DWORD     dwExStyle;
              }  CREATESTRUCT;  */

            hDC = GetDC(hWnd);
            GetTextMetrics(hDC, &tm);
            SetWindowPos(hWnd, -1, 0, 0,
                tm.tmMaxCharWidth * mgeom.winwid,
                (tm.tmHeight + tm.tmExternalLeading) * mgeom.winhgt,
                SWP_NOMOVE + SWP_NOZORDER);
            ReleaseDC(hWnd, hDC);
            break;

        case WM_DESTROY:
            KillTimer(hWnd, 1);
            PostQuitMessage(0);
            break;

        case WM_MOVE:
            break;

        case WM_SIZE:
            /* wParam contains a code indicating the requested sizing */
            cxClient = LOWORD(lParam);
            cyClient = HIWORD(lParam);
            /* lParam contains the new height and width of the client area */
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
                if (wasIconic) {
                    go_iconic();
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
            if (GetMenu(hWnd) == wParam) {
                CheckMenuItem(wParam, IDM_F_TESTMODE,
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

/*

    nCwRegisterClasses Function

    The following function  registers  all  the  classes  of  all  the
    windows associated with this application.  The function returns an
    error code if unsuccessful, otherwise it returns 0.

*/

int nCwRegisterClasses(void)
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
    wndclass.hIcon = NULL;                  /* Allow user-defined icon */
    wndclass.hCursor = LoadCursor(NULL, IDC_ARROW);
    /* Create brush for erasing background */
    wndclass.hbrBackground = (HBRUSH) (COLOR_WINDOW+1);
    wndclass.lpszMenuName = szAppName;      /* Menu Name is App Name */
    wndclass.lpszClassName = szAppName;     /* Class Name is App Name */
    if (!RegisterClass(&wndclass))
        return -1;
    return 0;
}
