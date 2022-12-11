#!/usr/bin/bash
export WINEDEBUG=-all
cd "$(dirname "$0")"
wine ./_thirdparty/mpasm/MPASMWIN.exe /q /p16F877A "ViyLineCAP.asm" /l"ViyLineCAP.lst" /e"ViyLineCAP.err" /d__DEBUG=1

# No need to scream
if [ -f ViyLineCAP.HEX ]; then
    mv ViyLineCAP.HEX ViyLineCAP.hex
fi
