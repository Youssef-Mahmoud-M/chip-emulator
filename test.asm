Interrupt 0:
IN R7
ADD R0 R0 R7
RTI
Interrupt 1:
; Multiples R7 by R6 and stores in R5
LDM R5 0

IADD R6 R6 -1
LDM R4 22
JN R4
ADD R5 R5 R7
LDM R4 12
JMP R4
RTI 
Main:
LDM R7 3
IN R6
INT 1
OUT R5
HLT
