opcodes:
```
0 0 0 0 0|S R C  LOADREG  R[SRC]->ACC
0 0 0 0 1|D S T  STOREREG  ACC->R[SRC]
0 0 0 1 *
0 0 1 * *
0 1 0 0 0|S R C  XOR  ACC^R[SRC]->ACC
0 1 0 0 1|S R C  AND  ACC&R[SRC]->ACC
0 1 0 1 0|S R C  OR  ACC|R[SRC]->ACC
0 1 0 1 1|S R C  ADD  ACC+R[SRC]->ACC
0 1 1 0 0|S R C  NOT  ~R[SRC]->ACC
0 1 1 0 1
0 1 1 1 0|S R C  LOADMEM MEM[R[SRC]]->ACC
0 1 1 1 1|D S T  STOREMEM ACC->MEM[R[SRC]]
1 0 0 0|I M M D  LOADLO SIGNEXT(IMMD)->ACC
1 0 0 1|I M M D  LOADHI (IMMD<<4)|(ACC&0xF)->ACC
1 0 1|I M M E D  JMP  PC+SIGNEXT(PC)->PC
1 1 0|I M M E D  JZ   PC+SIGNEXT(PC)->PC
1 1 1|I M M E D  JNZ  PC+SIGNEXT(PC)->PC
```

registers:
```
ACC
B
C
D
E
F
SP
PC
```

push r:
```
LOADLO -1
ADD SP
STOREREG SP
LOADREG r
STOREMEM [SP]
```

pop r:
```
LOADMEM [SP]
STOREREG r
LOADLO 1
ADD SP
STOREREG SP
```

call:
```
# decrement SP
LOADLO -1
ADD SP
STOREREG SP
# store return address
LOADLO 0x5
ADD PC
STOREMEM [SP]
# jump
LOADLO (ip&0xF)
LOADHI (ip>>4)
STOREREG PC
```

ret:
```
# jump to return address
LOADMEM [SP]
STOREREG PC
```

func:
```
# save registers we'll need (e.g. B, C)
push b
push c

pop c
pop b
LOADMEM [SP]
STOREREG PC
```

fib:
```
int fib(int n) {
    int sum;
    if (n == 0 || n == 1) {
        sum = n; //fib_block1
    } else {
        sum = fib(n-1); //fib_block2
        sum += fib(n-2);
    }
    return sum;
}

# B is first input and output
:fib
push C # C := int n 
push D # D := int sum
# initialize n
loadreg b
storereg c
# if n == 0 -> return n
loadreg c
jz :fib_block1 
# if n + -1 == 0 -> return n
loadlo -1 
add c
jz :fib_block1 
loadlo -1
add C
storereg b
call :fib
loadreg b
storereg d
loadlo -2
add c
storereg b
call :fib
loadreg b
add d
storereg d
jmp :fib_return
:fib_block1
loadreg c
storereg d
:fib_return 
# return sum
loadreg d
storereg b
# restore regs
pop d
pop c
ret
```
