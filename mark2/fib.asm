loadlo 2
call :fib

# int fib(int n) {
#     int sum;
#     if (n == 0 || n == 1) {
#       sum = n; //fib_block1
#     } else {
#       sum = fib(n-1); //fib_block2
#       sum += fib(n-2);
#     }
#     return sum;
# }

# B is first input and output
:fib
push c # C := int n 
push d # D := int sum
# initialize n
loadreg b
storereg c
# if n == 0 -> return n
loadreg c
jz :fib_block1 
# if n + -1 == 0 -> return n
loadlo f # -1 
add c
jz :fib_block1 
loadlo f # -1
add c
storereg b
call :fib
loadreg b
storereg d
loadlo e # -2
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