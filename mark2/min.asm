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
# Function: main
:main
# sp+4 -> RESULT
# sp+3 -> RETURN_ADDRESS
# sp+2 -> saved b
# sp+1 -> saved c
# sp+0 -> result
# save registers
push b
push c
# create stack space
loadlo  f
add sp
storereg sp
# Assign { local: "result", value: Operation(Add, Number(1), Number(1)) }
loadlo 00
add sp
storereg c
push c
loadlo 01
loadhi 00
storereg b
push b
loadlo 01
loadhi 00
storereg b
pop c
loadreg b
add b
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
jmp :main__EPILOGUE
:main__EPILOGUE
loadlo 1
add sp
storereg sp
pop c
pop b
ret
