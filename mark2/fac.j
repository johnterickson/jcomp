FUNCTION main() {
    CALL result := fac(5);
    RETURN result;
}

FUNCTION fac(n) {
    IF (n == 0) {
        RETURN 1;
    }
    CALL product := fac((n - 1));
    ASSIGN product := (n * product);
    RETURN product;
}