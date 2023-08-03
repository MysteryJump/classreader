package com.github.nreopigs.classreader;

import java.util.ArrayList;
import java.util.List;

public class ClassReader {
    private static native ArrayList extractFromJarPath(String jarPath);

    private static native String test(String a);

    static {
        var pwd = System.getProperty("user.dir");
        System.load(pwd + "/target/debug/classreader.dll");
    }

    public static void main(String[] args) {
        System.out.println(test("hello world"));

        List bytes = extractFromJarPath("C:\\Users\\nreop\\Desktop\\test.jar");
        List<Byte> b2 = bytes;
        byte[] b3 = new byte[bytes.size()];
        for (int i = 0; i < bytes.size(); i++) {
            b3[i] = b2.get(i);
        }
        System.out.println(new String(b3));

    }
}
