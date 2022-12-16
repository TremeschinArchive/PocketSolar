; |------------------------------------------------------------------| ;
;        | (c) 2022 Tremeschin, MIT License | ViyLine Project |        ;
; |------------------------------------------------------------------| ;

ERRORLEVEL -205, -207, -302, -203

; PIC16F877A Headers
list p=16f877a
#include <p16f877a.inc>

; Config word
__CONFIG _HS_OSC & _WDT_OFF & _PWRTE_ON & _BODEN_OFF & _LVP_OFF

; |------------------------------------------------------------------| ;

; Constants
Fosc	 equ .16
baudrate equ .9600

; Capacitor states on PORTB
capacitorChargeHex    equ B'00010001'
capacitorDischargeHex equ B'00100010'

; Reset vector, first instruction is goto setup
org         0x00
goto        setup

; Interruption -> Do nothing
org         0x04
goto        main

; |------------------------------------------------------------------| ;
; Macros / Syntactic sugars

; Switch to memory page 0
memoryPage0     macro
    bcf         STATUS,RP0
    bcf         STATUS,RP1
    endm

; Switch to memory page 1
memoryPage1     macro
    bsf         STATUS,RP0
    bcf         STATUS,RP1
    endm

; If file is equal to literal, execute next command
ifeq            macro file, literal
    movf        file,W
    xorlw       literal
    btfsc       STATUS,Z
    endm

; If file is not equal to literal, execute next command
ifneq           macro file, literal
    movf        file,W
    xorlw       literal
    btfss       STATUS,Z
    endm

; If file > literal, execute next command
ifgreater       macro file, literal
    movlw       literal
    subwf       file,W
    btfsc       STATUS,C
    endm

; Move literal to file
movlf           macro literal, file
    movlw       literal
    movwf       file
    endm

; Copy fileA to fileB
copy            macro fileA, fileB
    movf        fileA,W
    movwf       fileB
    endm

; Transmit a literal
txliteral       macro literal
    movlw       literal
    call        TxCarUART
    endm

; Notify Rust PIC is busy
notifyBusy      macro
    txliteral   0x00
    endm

; Notify Rust PIC is ready for command
notifyFree      macro
    txliteral   0xFF
    endm

; |------------------------------------------------------------------| ;
; Setup

setup:
    memoryPage1
        ; RB0 and RB1 control the capacitor
        clrf        TRISB

        ; AN0, AN1, AN3 as analog
        movlf       0x84,ADCON1

    ; Clear measurements and dischage capacitor default state
    memoryPage0
	    call	    setupUART
        call        clearMeasurements
        call        dischargeCapacitor

    goto        main

; |------------------------------------------------------------------| ;
; Main

cblock 0x20
    ; The received data instruction from the outside world
    rxdata
endc

main:
    ; Always the default state is discharging
    movlf       capacitorDischargeHex,PORTB

    ; Read some data from UART loop
    mainWaitRxLoop:
        call        RxCarUART
	    btfss	    flag_rx,0
	    goto	    mainWaitRxLoop

    ; Store received data
    movwf       rxdata

    ; Code 2 -> Send next 8 bits from read values
    ifeq        rxdata,0x01
    call        sendNextByte

    ; Code > 2 -> Make measurement with Delta T = $command
    ifgreater   rxdata,0x02
    call        measureFull

    goto        main

; |------------------------------------------------------------------| ;
; Measurement routines

cblock
    measureDelays
endc

; Do a single measurement
singleMeasure:
    copy        rxdata,measureDelays
    call        pointCollectedNotification

    ; Call measureDelay $rxdata times
    singleMeasureLoop:
        call        delay1ms
        decf        measureDelays,F
        ifneq       measureDelays,0x0
        goto        singleMeasureLoop

    ; Measure voltage and current
    call        readVoltage
    call        readCurrent
    return

; Make a full measurement (Measure multiple times)
measureFull:
    notifyBusy
    call        dischargeCapacitor
    call        clearMeasurements
    call        chargeCapacitor
    measureLoop:
        call        singleMeasure
        ifneq       FSR,0x80
        goto        measureLoop
    call        resetMeasurmentsPointer
    notifyFree
    return

; Send some external signal saying we collected a point
pointCollectedNotification:
    btfsc       PORTB,6
    goto        $ + 3
        bsf         PORTB,6
        return
    bcf         PORTB,6
    return

; - - - - - - - - - - - - - - - - - - - ;
; Capacitor functions

chargeCapacitor:
    movlf       capacitorChargeHex,PORTB
    return

dischargeCapacitor:
    movlf       capacitorDischargeHex,PORTB
    call        delay500ms
    return

; - - - - - - - - - - - - - - - - - - - ;
; Measurement data manipulation

; Clear all measurements made
clearMeasurements:
    call        resetMeasurmentsPointer
    clearLoop:
        clrf        INDF
        incf        FSR,F
        ifneq       FSR,0x80
        goto        clearLoop
    call        resetMeasurmentsPointer
    return

; Reset FSR to the starting value of measurements
resetMeasurmentsPointer:
    movlf       0x30,FSR
    return

; Send the next pack of 8 bits to the outside world
sendNextByte:
    movf        INDF,W
	call        TxCarUART
    incf        FSR,F
    return

; - - - - - - - - - - - - - - - - - - - ;
; ADC functions

cblock
    minimumAquisitionTime
endc

; Voltage is read on RA1 == AN1 port, configure muxer and read
readVoltage:
    movlf       0x89,ADCON0
    goto        _readADC

; Current is read on RA3 == AN3 port, configure muxer and read
readCurrent:
    movlf       0x99,ADCON0
    goto        _readADC

; Read the input analog signal, demuxer configured previously
_readADC:

    ; Wait minimum acquisition time (20us)
    movlf       0x1A,minimumAquisitionTime
    decfsz      minimumAquisitionTime,F
    goto        $ - 1

    ; Start and wait done conversion
    bsf         ADCON0,GO_DONE
    btfsc       ADCON0,GO_DONE
    goto        $ - 1

    ; Save measurements into FSR
    saveMeasurements:
        memoryPage0
        copy        ADRESH,INDF
        incf        FSR,F

        memoryPage1
        movf        ADRESL,W
        memoryPage0
        movwf       INDF
        incf        FSR,F

    return

; |------------------------------------------------------------------| ;
; Delays for 20 MHz

cblock
    delayCounter1
    delayCounter2
    delayCounter3
endc

; 1 ms delay function
delay1ms:
    movlf       0x0C,delayCounter1
    movlf       0x0D,delayCounter2
    delay1msLoop:
        decfsz      delayCounter1,F
        goto        delay1msLoop
        decfsz      delayCounter2,F
        goto        delay1msLoop
    return

; 500 ms delay function
delay500ms:
    movlf       0x5C,delayCounter1
    movlf       0x26,delayCounter2
    movlf       0x0B,delayCounter3
    delay500msLoop:
        decfsz      delayCounter1,F
        goto        delay500msLoop
        decfsz      delayCounter2,F
        goto        delay500msLoop
        decfsz      delayCounter3,F
        goto        delay500msLoop
    return

; |------------------------------------------------------------------| ;

#include "UART.asm"

END
