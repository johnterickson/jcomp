# set stack to 0xff
loadlo f
storereg sp
# call main
loadlo f
add sp
storereg sp
call :main
pop b
halt
# Function: fib
:fib
# sp+4 -> RESULT
# sp+3 -> n
# sp+2 -> RETURN_ADDRESS
# sp+1 -> sum1
# sp+0 -> sum2
# create stack space
loadlo  e
add sp
storereg sp
# If { predicate: Operation(Equals, Ident("n"), Number(0)), when_true: [Return { value: Number(1) }] }
loadlo 03
add sp
loadmem acc
storereg b
push b
loadlo 00
loadhi 00
storereg b
pop c
loadreg b
xor c
storereg b
loadreg b
jnz :fib__function0__IF_SKIP
# Return { value: Number(1) }
loadlo 01
loadhi 00
storereg b
loadlo 04
add sp
storereg c
loadreg b
storemem c
loadlo :fib__EPILOGUE
loadhi :fib__EPILOGUE
storereg pc
:fib__function0__IF_SKIP
# If { predicate: Operation(Equals, Ident("n"), Number(1)), when_true: [Return { value: Number(1) }] }
loadlo 03
add sp
loadmem acc
storereg b
push b
loadlo 01
loadhi 00
storereg b
pop c
loadreg b
xor c
storereg b
loadreg b
jnz :fib__function1__IF_SKIP
# Return { value: Number(1) }
loadlo 01
loadhi 00
storereg b
loadlo 04
add sp
storereg c
loadreg b
storemem c
loadlo :fib__EPILOGUE
loadhi :fib__EPILOGUE
storereg pc
:fib__function1__IF_SKIP
# Call { local: "sum1", function: "fib", parameters: [Operation(Subtract, Ident("n"), Number(1))] }
dec sp
loadlo 04
add sp
loadmem acc
storereg b
push b
loadlo 01
loadhi 00
storereg b
pop c
not b
storereg b
loadlo 1
add b
add c
storereg b
push b
call :fib
loadlo 01
add sp
storereg sp
pop b
loadlo 01
add sp
storereg c
loadreg b
storemem c
# Call { local: "sum2", function: "fib", parameters: [Operation(Subtract, Ident("n"), Number(2))] }
dec sp
loadlo 04
add sp
loadmem acc
storereg b
push b
loadlo 02
loadhi 00
storereg b
pop c
not b
storereg b
loadlo 1
add b
add c
storereg b
push b
call :fib
loadlo 01
add sp
storereg sp
pop b
loadlo 00
add sp
storereg c
loadreg b
storemem c
# Return { value: Operation(Add, Ident("sum1"), Ident("sum2")) }
loadlo 01
add sp
loadmem acc
storereg b
push b
loadlo 01
add sp
loadmem acc
storereg b
pop c
loadreg b
add c
storereg b
loadlo 04
add sp
storereg c
loadreg b
storemem c
loadlo :fib__EPILOGUE
loadhi :fib__EPILOGUE
storereg pc
:fib__EPILOGUE
loadlo 2
add sp
storereg sp
ret
# Function: main
:main
# sp+2 -> RESULT
# sp+1 -> RETURN_ADDRESS
# sp+0 -> result
# create stack space
loadlo  f
add sp
storereg sp
# Call { local: "result", function: "fib", parameters: [Number(5)] }
dec sp
loadlo 05
loadhi 00
storereg b
push b
call :fib
loadlo 01
add sp
storereg sp
pop b
loadlo 00
add sp
storereg c
loadreg b
storemem c
# Return { value: Ident("result") }
loadlo 00
add sp
loadmem acc
storereg b
loadlo 02
add sp
storereg c
loadreg b
storemem c
loadlo :main__EPILOGUE
loadhi :main__EPILOGUE
storereg pc
:main__EPILOGUE
loadlo 1
add sp
storereg sp
ret
