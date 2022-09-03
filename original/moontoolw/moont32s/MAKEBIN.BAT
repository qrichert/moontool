@rem
@rem    Make release binary archives
@rem
del %TEMP%\moont32.zip 
pkzip %TEMP%\moont32.zip Release\Moontool.exe Moontool.hlp
