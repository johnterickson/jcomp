FUNCTION main() {
    CALL result := fib(5);
    RETURN result;
}

FUNCTION fib(n) {
    IF (n == 0) {
        RETURN 1;
    }
    ASSIGN n := (n - 1);
    IF (n == 0) {
        RETURN 1;
    }
    CALL sum1 := fib(n);
    ASSIGN n := (n - 1);
    CALL sum2 := fib(n);
    RETURN (sum1 + sum2);
}