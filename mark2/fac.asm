loadlo 5
storereg b
call :fac
loadlo f
loadhi f
storereg pc
# int fac(int n #b# ) {
#     int product; # c
#     if (n == 0) {
#       product = 1;
#     } else {
#       product = n;
#       product *= fac(n-1); //:recurse
#     }
#     :prologue
#     return sum;
# }
:fac
push c -> 253, 251; ra @ 254, 252
loadreg b
jnz :recurse
loadlo 1
storereg c
loadlo :prologue
loadhi :prologue
storereg pc
:recurse
loadreg b
storereg c # product = n; 
loadlo f
add b
storereg b
call :fac
loadreg b
mul c
storereg c
:prologue
loadreg c
storereg b
pop c
ret