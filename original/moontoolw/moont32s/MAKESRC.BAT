@rem
@rem Make source release archive
@rem
del %TEMP%\moont32s.zip 
call clean
pkzip -pr %TEMP%\moont32s.zip *.*
