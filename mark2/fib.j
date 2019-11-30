FUNCTION main() {
    CALL RESULT := fib(5);
    RETURN;
}

FUNCTION fib(n) {
    IF (n == 0) {
        RESULT := 1;
        RETURN;
    }
    IF (n == 1) {
        RESULT := 1;
        RETURN;
    }
    CALL RESULT := fib((n - 1));
    CALL sum2 := fib((n - 2));
    RESULT := (RESULT + sum2);
    RETURN;
}