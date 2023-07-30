class Main<T extends String, E> implements AA {
    public java.util.List<? extends Main> mai;
    public java.util.List<T> mai2;
    public java.util.Map<E, ? super T> mai3;
    public int test;

    public Integer test2;
    public T test3;
    public int[][] test4;

    public java.util.List<T>[][][] test5;

    public <R, S extends java.util.List & AA> int add(int a, int b) {
        return a + b;
    }
}

class Main2 extends java.util.ArrayList implements AA {

}

interface AA {

}