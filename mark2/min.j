FUNCTION main() {
    CALL result := adder(4,5);
    RETURN result;
}

FUNCTION adder(x,y) {
    ASSIGN result := (x + y);
    RETURN result;
}