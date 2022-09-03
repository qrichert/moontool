@rem
@rem    Make release binary archives
@rem
del %TEMP%\moont16.zip 
pkzip %TEMP%\moont16.zip Moontool.exe Moontool.txt
