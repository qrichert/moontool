rem
rem Clean up non-source files
rem
@rem Note: this script uses Metaware's emulation
@rem       (more or less) of Unix "find", not the
@rem       standard MS-DOS FIND.
@rem
find . -name *.exe!*.res!*.pch!*.bsc!*.obj!*.pdb!*.sbr -rm
find . -name *.wsp!*.vcw!*.aps!*.clw!*.bak!*.lib!*.map -rm
find . -name *.fts!*.idb!*.ilk!*.sup!*.exp             -rm
del *.hlp
del *.sfx
find help -name *.log!*.ph!~*.rtf                      -rm
