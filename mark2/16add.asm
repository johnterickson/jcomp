# set up stack pointer
loadlo f
loadhi 0
storereg sp
# prep args
loadlo 0
storereg b
push b #s1
push b #s0
loadlo f # 255
storereg b
push b #y0
loadlo 2 # 2
storereg b
push b #x0
call :u8_add_with_carry
# skip two inputs
loadlo 2
add sp
storereg sp
# pop off result
loadmem sp
storereg c
inc sp
loadmem sp
storereg b
halt



:u8_add_with_carry
# sp+0 saved b
# sp+1 saved c
# sp+2 rip
# sp+3 x0
# sp+4 y0
# sp+5 s0
# sp+6 s1
push c
push b
loadlo 5
add sp
storereg c # c: &s0
loadlo 3
add sp
loadmem acc
storereg b # b: x0
loadlo 4
add sp
loadmem acc # a:y0
add b # a:x0+y0 FLAGS has carry
storemem c # save x0+y0 to s0
loadlo 1
and flags
jz :no_carry
loadlo 1
jmp :save_carry
:no_carry
loadlo 0
:save_carry
storereg b
loadlo 6
add sp
storereg c
loadreg b
storemem c
pop b
pop c
ret