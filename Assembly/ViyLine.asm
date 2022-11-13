; | (c) 2022 Tremeschin, MIT License | ViyLine Project | ;
list      p=16f877a
#include <p16f877a.inc>

; |------------------------------------------------------------------| ;

banco0	macro
	bcf	STATUS,RP0
	bcf	STATUS,RP1
	endm

banco1	macro
	bsf	STATUS,RP0
	bcf	STATUS,RP1
	endm

; |------------------------------------------------------------------| ;

__CONFIG _HS_OSC & _WDT_OFF & _PWRTE_ON & _BODEN_OFF & _LVP_OFF
; __CONFIG H'2F02'		; Palavra de configura��o
; ERRORLEVEL -305, -302


; General Purpose Registers from 0x20 address onwards
cblock 0x20
	conta_ad, Binario10H, Binario10L
endc

; Reset vector, first instruction is goto setup
org		0x00
goto	setup

; Empty but reserved interruption vector
org		0x04

; |------------------------------------------------------------------| ;

setup:
	banco1
    ; AN0, AN1, AN3 as analog
    movlw	0x84
    movwf	ADCON1
	banco0
    goto main

; |------------------------------------------------------------------| ;

; Voltage is read on RA1 == AN1 port, configure muxer and read
readVoltage:
	movlw	0x89
	movwf	ADCON0
    goto    _readADC

; Current is read on RA3 == AN3 port, configure muxer and read
readCurrent:
	movlw	0x99
	movwf	ADCON0
    goto    _readADC

; Read the input analog signal, demuxer configured previously
_readADC:
	movlw	.26				;Tadq >= 20us (Fclock = 16MHz)
	movwf	conta_ad
	decfsz	conta_ad,F		;(3N+3)c   (inclui call)
	goto	$ - 1			;aguarda tempo de aquisi��o

    ; Start the Analog to Digital conversion
	bsf		ADCON0,GO_DONE

    ; Wait until the conversion is finished
	btfsc 	ADCON0,GO_DONE
    goto	$ - 1

    ; Read value to W
	movf	ADRESH,W
	movwf	Binario10H
	banco1
	movf	ADRESL,W
	banco0
	movwf	Binario10L   	;Resultado em Binario10
	return

; |------------------------------------------------------------------| ;

cblock

endc

main:
    call    delay_1s    ; FIXME: Sincronizar com PWM
	call	readCurrent
    ; Envia Binario10L, Binario10H

    call    delay_1s    ; FIXME: Sincronizar com PWM
	call	readVoltage
    ; Envia Binario10L, Binario10H

    goto main


; |------------------------------------------------------------------| ;

cblock
    contador1
    contador2
    contador3
endc

delay_500ms:
	movlw	D'92'
	movwf	contador1
	movlw	D'38'
	movwf	contador2
	movlw	D'11'
	movwf	contador3
	decfsz	contador1,F	;
	goto	$-1
	decfsz	contador2,1
	goto	$-3
	decfsz	contador3,1
	goto	$-5
	return

delay_1s:
	movlw	D'189'
	movwf	contador1
	movlw	D'75'
	movwf	contador2
	movlw	D'21'
	movwf	contador3
	decfsz	contador1,1
	goto	$-1
	decfsz	contador2,1
	goto	$-3
	decfsz	contador3,1
	goto	$-5
	return

; |------------------------------------------------------------------| ;


	END