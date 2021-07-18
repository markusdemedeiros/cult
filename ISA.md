ISA for the program


- Operates on 32 bit numbers/ 4 byte ABCD
- 8 registers
- Arrays of 32 bit numbers, indexed by 32 bit number
- Program stored in array labelled 0
- Capable of unisgned 8 bit ASCII IO
- Registers initialized to zero
- Execution starts on first platter of zero array


14 instructions

Operator number is first 4 bits
A/B/C are the last 9 bits

0	CMOV
==========
	if C != 0
		A <- B	

1	LOAD
==========
	A <- ARRAY(B)[C]	

2	STORE
==========
	ARRAY(A)[B] <- C

3	ADD
==========
	A <- B + C % 2^32

4	MUL
==========
	A <- B * C % 2^32

5	DIV
==========
	A <- B / C % 2^32
	- round down?

6 	NAND
==========
	bitwiseif B = 0 | C = 0
		A <- 1
	else
		A <- 0

7	HALT
==========
	Stop machine

8	ALLOC
==========
	Allocate array with C 
	platters of zeroes
	B <- Array label

9	FREE
==========
	Discard array C

A	OUT
==========
	Console output C, 0-255 allowed

B	IN
==========
	C <- Console input 
	Signed 8 bit number extended to 32 bit

C	CALl
==========
	Array B cloned and replaces 0 array
	Execution is placed at line C

D	CONST
==========
	A is the next 3 bits after the operator
	value is the remaining bits
	A <- value

FAULTS/EXCEPTIONSored at offset
                  in register C in the array identified by B.

	BAD INSTRUCTION:
	INDEX OR APPEND TO INACTIVE ARRAY
	FREE 0
	DOUBLE FREE
	DIVIDE 0
	CALL INACTIVE
	OUT > 255
	EXECUTION RUNS OFF OF PLATER		




	
