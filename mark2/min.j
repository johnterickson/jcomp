FUNCTION main() {
    CALL result := increment(5);
    RETURN result;
}

FUNCTION increment(n) {
    ASSIGN result := (n + 1);
    RETURN result;
}