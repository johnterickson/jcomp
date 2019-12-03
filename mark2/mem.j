FUNCTION main() {
    ASSIGN x := 255;
    STORE x -> *0;
    LOAD x <- *0;
    RETURN x;
}