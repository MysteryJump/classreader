class Main<T extends String, E> {
    public java.util.List<? extends Main> mai;
    public java.util.List<T> mai2;
    public java.util.Map<E, ? super T> mai3;

    public <R, S extends java.util.List & AA> int add(int a, int b) {
        return a + b;
    }
}

interface AA {

}