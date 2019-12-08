FUNCTION main() {
    CALL result := fib(13);
    RETURN result;
}

FUNCTION fib(n) {
    IF (n == 0) {
        RETURN 0;
    }
    IF (n == 1) {
        RETURN 1;
    }
    ASSIGN addr := 0;
    ASSIGN addr := (addr + n);
    LOAD sum1 <- *addr;
    IF (sum1 == 0) {
        CALL sum1 := fib((n - 1));
        CALL sum2 := fib((n - 2));
        ASSIGN sum1 := (sum1 + sum2);
        STORE sum1 -> *addr;
    }
    RETURN sum1;
}