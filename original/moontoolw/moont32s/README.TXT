This program was last built with Microsoft Visual C++ (all
code is pure C, no C++ gnarl) 5.0 and tested under Bounds
Checker 5.03.  Visual Studio workspace and project files are
included.  If you rebuild using a different compiler, you'll
need to manually create a Makefile or whatever form of
description that compiler requires, naming all the .C files
and the .RC files as components.

Oh yes, you ask, why did he set optimisation for the Release
build to "Minimize Size" rather than "Maximize Speed"?  Because
brain-dead Monkey C *hangs* while compiling Mooncalc.c if
you select "Maximize Speed"!  Kode Kiddies never, ever get
the (floating) point.

For additional information about this program, please consult
the Moontool.hlp file or visit the Web page:

	http://www.fourmilab.ch/moontoolw/

John Walker
http://www.fourmilab.ch/

March MIM
