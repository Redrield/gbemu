; Reads a string from HL and sends it byte-by-byte over SB until $00 is read.
; CLOBBERS: HL,A
send_string:
    .loop:
        ldi A,(HL)
        cp A,0
        ret z
        call send_byte
        jp .loop

; Sends a byte over the Serial Bus to the emulator. Byte should be valid ascii stored in A
; CLOBBERS: A
send_byte:
    ld ($FF01),A
    ld A,$81
    ld ($FF02),A
    ld A,$00
    ld ($FF02),A
    ret

