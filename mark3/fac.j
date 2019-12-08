FUNCTION main() {
    CALL result := fac(5);
    RETURN result;
}

FUNCTION fac(n) {
    IF (n == 0) {
        RETURN 1;
    }
    ASSIGN addr := 0;
    ASSIGN addr := (addr + n);
    LOAD product <- *addr;
    IF (product == 0) {
        CALL product := fac((n - 1));
        ASSIGN product := (n * product);
        STORE product -> *addr;
    }
    RETURN product;
}