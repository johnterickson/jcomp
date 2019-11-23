FUNCTION main() {
    CALL result := fac(5);
    RETURN result;
}

FUNCTION fac(n) {
    IF (n == 0) {
        RETURN n;
    }
    ASSIGN nMinus1 := (n - 1);
    CALL product := fac(nMinus1);
    ASSIGN product := (n * product);
    RETURN n;
}