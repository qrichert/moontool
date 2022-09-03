@rem
@rem Make source release archive
@rem
del %TEMP%\moont16s.zip 
del *.obj
del *.bak
pkzip -pr %TEMP%\moont16s.zip *.*
