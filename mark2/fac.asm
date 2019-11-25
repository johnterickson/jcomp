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
# Function: fac
:fac
# sp+3 -> RESULT
# sp+2 -> n
# sp+1 -> RETURN_ADDRESS
# sp+0 -> product
# create stack space
loadlo  f
add sp
storereg sp
# If { predicate: Operation(Equals, Ident("n"), Number(0)), when_true: [Return { value: Number(1) }] }
loadlo 02
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
jnz :fac__function0__IF_SKIP
# Return { value: Number(1) }
loadlo 01
loadhi 00
storereg b
loadlo 03
add sp
storereg c
loadreg b
storemem c
loadlo :fac__EPILOGUE
loadhi :fac__EPILOGUE
storereg pc
:fac__function0__IF_SKIP
# Call { local: "product", function: "fac", parameters: [Operation(Subtract, Ident("n"), Number(1))] }
dec sp
loadlo 03
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
call :fac
loadlo 01
add sp
storereg sp
pop b
loadlo 00
add sp
storereg c
loadreg b
storemem c
# Assign { local: "product", value: Operation(Multiply, Ident("n"), Ident("product")) }
loadlo 02
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
mul c
storereg b
loadlo 00
add sp
storereg c
loadreg b
storemem c
# Return { value: Ident("product") }
loadlo 00
add sp
loadmem acc
storereg b
loadlo 03
add sp
storereg c
loadreg b
storemem c
loadlo :fac__EPILOGUE
loadhi :fac__EPILOGUE
storereg pc
:fac__EPILOGUE
loadlo 1
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
# Call { local: "result", function: "fac", parameters: [Number(5)] }
dec sp
loadlo 05
loadhi 00
storereg b
push b
call :fac
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
