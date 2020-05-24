main:
    org $100
    LD HL,tile
    PUSH HL
    LD HL,$8000
    ; Transfer the tile data into the tile map
    .tx_loop:
        ; Backup pointer to tilemap in BC
        PUSH HL
        POP BC
        ; Get pointer into tile data in code
        POP HL
        ; Load value, increment HL
        LDI A,(HL)
        ; Push code tile data addr onto stack
        PUSH HL
        ; Reload backup, push next byte onto tile map
        PUSH BC
        POP HL
        LDI (HL),A
        ; Break if it was a stop byte
        cp A,$ff
        JP nz,.tx_loop


    ; Set up palette
    LD A,%11100100
    LD ($FF47),A
    ; Enable LCD
    LD A,%10010000
    LD ($FF40),A
    .loop:
        nop
        jp .loop

tile: db $81, $00, $00, $42, $3c, $3c, $3c, $3c, $3c, $3c, $3c, $3c, $00, $42, $81, $00, $ff
