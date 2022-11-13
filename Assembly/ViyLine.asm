; | (c) 2022 Tremeschin, MIT License | ViyLine Project | ;
list p=16f877a
#include <p16f877a.inc>
__CONFIG _HS_OSC & _WDT_OFF & _PWRTE_ON & _BODEN_OFF & _LVP_OFF
ERRORLEVEL -305, -302

; |------------------------------------------------------------------| ;

memoryPage0	macro
	bcf	STATUS,RP0
	bcf	STATUS,RP1
	endm

memoryPage1	macro
	bsf	STATUS,RP0
	bcf	STATUS,RP1
	endm

; |------------------------------------------------------------------| ;

; General Purpose Registers from 0x20 address onwards
cblock 0x20

	; Analog to Digital Converter related
	Binario10H, ; Upper 8-bits of the measured analog signal
	Binario10L, ; Lower 8-bits of the measured analog signal
	conta_ad,

	; Duty-time / cycles related
	dutyTime,   ; Percentage of ON state (relative to 255)
	dutyStep,   ; At this t+dt, are we ON or OFF (compare to dutyTime)
	cycle,      ; How many cycles have passed for the inductor to stabilize?
	measuring   ; Are we measuring voltage or current? Checks bit 0 only

endc

; Reset vector, first instruction is goto setup
org		0x00
goto	setup

; Empty but reserved interruption vector
org		0x04
goto	interruption

; |------------------------------------------------------------------| ;

setup:

	memoryPage1
		; AN0, AN1, AN3 as analog
		movlw	0x84
		movwf	ADCON1
		clrf	TRISB

		; Timer0 Prescaler
		bsf		OPTION_REG,PS0
		bcf		OPTION_REG,PS1
		bcf		OPTION_REG,PS2
		bcf		OPTION_REG,PSA
		bcf		OPTION_REG,T0CS

		; Enable Global Interruptions
		bsf		INTCON,GIE

		; Enable interruption due Timer 0
		bsf		INTCON,T0IE

	memoryPage0
		clrf	dutyTime
		clrf	dutyStep
		clrf	TMR0

	goto 	main

; The code is based on interruptions since we get time precision there
main:
    nop
    goto 	main


; |------------------------------------------------------------------| ;

interruption:

	; Clear interruption flag
	bcf		INTCON,T0IF
	incf	dutyStep,F

	; Check if dutyStep is bigger or smaller than dutyTime
	; and apply the zero or one logical voltage
	defineMosfetState:

		; Applies
		movf	dutyStep,W
		subwf	dutyTime,W

		; If smaller, set zero
		btfss	STATUS,C
		bcf		PORTB,0

		; If bigger, set one
		btfsc	STATUS,C
		bsf		PORTB,0

	; Check if we are on a start duty
	ifFinishedLoop:
		movf	dutyStep,W
		xorlw	0xFF
		btfss	STATUS,Z
		retfie

	; We measure after 100 cycles, return if cycle isn't 100
	continueTooFewCycles:
		incf	cycle,F
		movf	cycle,W
		xorlw	D'5'
		btfss	STATUS,Z
		retfie
		clrf	cycle

	; Swap what we are measuring (current / voltage)
	incf	measuring,F

	; Measure voltage first
	btfss	measuring,0
	call	readVoltage

	; Measure current, increasy duty time
	btfss	measuring,0
	retfie
	call	readCurrent
	incf	dutyTime,F

	; TODO: Send the data to bluetooth

	retfie

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
	movlw	.26				; Tadq >= 20us (Fclock = 16MHz)
	movwf	conta_ad
	decfsz	conta_ad,F		; (3N+3)c   (inclui call)
	goto	$ - 1			; aguarda tempo de aquisi��o

    ; Start the Analog to Digital conversion
	bsf		ADCON0,GO_DONE

    ; Wait until the conversion is finished
	btfsc 	ADCON0,GO_DONE
    goto	$ - 1

    ; Read value to W
	movf	ADRESH,W
	movwf	Binario10H
	memoryPage1
	movf	ADRESL,W
	memoryPage0
	movwf	Binario10L   	;Resultado em Binario10
	return

; |------------------------------------------------------------------| ;

END
