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
# Function: increment
:increment
# sp+4 -> RESULT
# sp+3 -> n
# sp+2 -> RETURN_ADDRESS
# sp+1 -> saved c
# sp+0 -> result
# save registers
push c
# create stack space
loadlo  f
add sp
storereg sp
# Assign { local: "result", value: Operation(Add, Ident("n"), Number(1)) }
loadlo 00
add sp
storereg c
push c
loadlo 04
add sp
loadmem acc
storereg b
push b
loadlo 01
loadhi 00
storereg b
pop c
loadreg b
add c
storereg b
pop c
loadreg b
storemem c
# Return { local: "result" }
loadlo 04
add sp
storereg b
loadlo 00
add sp
storereg c
loadmem c
storemem b
jmp :increment__EPILOGUE
:increment__EPILOGUE
loadlo 1
add sp
storereg sp
pop c
ret
# Function: main
:main
# sp+3 -> RESULT
# sp+2 -> RETURN_ADDRESS
# sp+1 -> saved c
# sp+0 -> result
# save registers
push c
# create stack space
loadlo  f
add sp
storereg sp
# Call { local: "result", function: "increment", parameters: [Number(5)] }
dec sp
loadlo 05
loadhi 00
storereg b
push b
call :increment
loadlo 01
add sp
storereg sp
pop b
loadlo 00
add sp
storereg c
loadreg b
storemem c
# Return { local: "result" }
loadlo 03
add sp
storereg b
loadlo 00
add sp
storereg c
loadmem c
storemem b
jmp :main__EPILOGUE
:main__EPILOGUE
loadlo 1
add sp
storereg sp
pop c
ret
