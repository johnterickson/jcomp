# set up stack pointer
loadlo f
storereg sp
:test1
# store a value to memory
loadlo 1
storemem b
# clear
xor acc
# retrieve it
loadmem b
storereg c
loadlo f
add c
jz :test2
loadlo 1
halt
:test2
# todo

:success
halt