#!/usr/bin/bash
export WINEDEBUG=-all
cd "$(dirname "$0")"
wine ./thirdparty/mpasm/MPASMWIN.exe /q /p16F877A "ViyLineCAP.asm" /l"ViyLineCAP.lst" /e"ViyLineCAP.err" /d__DEBUG=1

# Remove nobody-asked-for-files
# rm ViyLine.err
# rm ViyLine.lst
# rm ViyLine.O

# No need to scream
# mv ViyLineCAP.HEX ViyLineCAP.hex
