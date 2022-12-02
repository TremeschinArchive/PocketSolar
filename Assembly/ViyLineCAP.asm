; | (c) 2022 Tremeschin, MIT License | ViyLine Project | ;
list p=16f877a
#include <p16f877a.inc>
__CONFIG _HS_OSC & _WDT_OFF & _PWRTE_ON & _BODEN_OFF & _LVP_OFF
ERRORLEVEL -305, -302

; Constants
Fosc	 equ .20
baudrate equ .9600

; Capacitor states on PORTB
capacitorDoNothingHex equ 0x00
capacitorChargeHex    equ 0x01
capacitorDischargeHex equ 0x02

; Reset vector, first instruction is goto setup
org         0x00
goto        setup

; Interruption -> Do nothing
org         0x04


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


; |------------------------------------------------------------------| ;
; Setup

setup:
    memoryPage1
        ; RB0 and RB1 control the capacitor
        movlf       0x03,TRISB

        ; AN0, AN1, AN3 as analog
        movlf       0x84,ADCON1
        clrf        TRISB

    memoryPage0
        call        clearMeasurements

	call	    setupUART
    goto        main


; |------------------------------------------------------------------| ;
; Main

cblock 0x20
    ; The received data instruction from the outside world
    rxdata
endc

main:
    ; Read some data from UART loop
    call        RxCarUART
	btfss	    flag_rx,0
	goto	    $ - .2

    ; Store received data
    movwf       rxdata

    ; Code 0 -> Clear measurements
    ifeq        rxdata,0x0
    call        clearMeasurements

    ; Code 1 -> Reset pointer to send data
    ifeq        rxdata,0x1
    call        resetMeasurmentsPointer

    ; Code 2 -> Send next 8 bits from read values
    ifeq        rxdata,0x2
    call        sendNextByte

    ; Code > 2 -> Make measurement with Delta T = $command
    ifgreater   rxdata,0x2
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

    ; Call measureDelay $rxdata times
    singleMeasureLoop:
        call        delay1ms
        decf        measureDelays,F
        ifneq       measureDelays,0x0
        goto        singleMeasureLoop

    ; Measure voltage and current
    call        readVoltage
    Call        readCurrent
    return

; Measure multiple times
measureFull:
    call        dischargeCapacitor
    call        chargeCapacitor
    measureLoop:
        call        singleMeasure
        ifneq       FSR,0x80
        goto        measureLoop
    movlf       capacitorDoNothingHex,PORTB
    return

; - - - - - - - - - - - - - - - - - - - ;
; Capacitor functions

chargeCapacitor:
    movlf       capacitorChargeHex,PORTB
    return

dischargeCapacitor:
    movlf       capacitorDischargeHex,PORTB
    call        delay500ms
    movlf       capacitorDoNothingHex,PORTB


; - - - - - - - - - - - - - - - - - - - ;
; Measurement data manipulation

; Reset FSR to the starting value of measurements
resetMeasurmentsPointer:
    movlf       0x30,FSR
    return

; Clear all measurements made
clearMeasurements:
    call        resetMeasurmentsPointer
        clrf        INDF
        incf        FSR,F
        ifneq       FSR,0x70
        goto        $ - .3
    call        resetMeasurmentsPointer
    return

; Send the next pack of 8 bits to the outside world
sendNextByte:
    movf        FSR,W
	call        TxCarUART
    incf        FSR,F
    return

; - - - - - - - - - - - - - - - - - - - ;
; ADC functions

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
    movlw       .26
    decfsz      W,F
    goto        $ - 1

    ; Start and wait done conversion
    bsf         ADCON0,GO_DONE
    btfsc       ADCON0,GO_DONE
    goto        $ - 1

    ; Save measurements into FSR
    memoryPage0
    movf        ADRESH,W
    movwf       INDF
    incf        FSR,F

    memoryPage1
    movf        ADRESL,W
    memoryPage0
    movwf       INDF
    incf        FSR,F
    return

; |------------------------------------------------------------------| ;
; Delays

cblock
    delayCounter1
    delayCounter2
    delayCounter3
endc

; 1 ms delay function
delay1ms:
    movlf       0x7B,delayCounter1
    movlf       0x07,delayCounter2
    delay1msLoop:
        decfsz      delayCounter1, 1
        goto        delay1msLoop
        decfsz      delayCounter2, 1
        goto        delay1msLoop
    return

; 500 ms delay function
delay500ms:
    movlf       0xB5,delayCounter1
    movlf       0xAF,delayCounter2
    movlf       0x0D,delayCounter3
    delay500msLoop:
        decfsz      delayCounter1, 1
        goto        delay500msLoop
        decfsz      delayCounter2, 1
        goto        delay500msLoop
        decfsz      delayCounter3, 1
        goto        delay500msLoop
    return

; |------------------------------------------------------------------| ;

#include "UART.asm"

END
