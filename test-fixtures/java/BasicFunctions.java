package testfixtures;

import java.util.*;
import java.util.function.*;
import java.util.stream.Collectors;
import java.util.concurrent.CompletableFuture;
import java.io.IOException;

public class BasicFunctions {
    
    // Basic method definitions
    public static String greet(String name) {
        return "Hello, " + name + "!";
    }
    
    public static int add(int a, int b) {
        return a + b;
    }
    
    // Method overloading
    public static double add(double a, double b) {
        return a + b;
    }
    
    public static String add(String a, String b) {
        return a + b;
    }
    
    // Methods with different access modifiers
    public void publicMethod() {
        System.out.println("Public method");
    }
    
    private void privateMethod() {
        System.out.println("Private method");
    }
    
    protected void protectedMethod() {
        System.out.println("Protected method");
    }
    
    void packagePrivateMethod() {
        System.out.println("Package private method");
    }
    
    // Static methods
    public static void staticMethod() {
        System.out.println("Static method");
    }
    
    // Final methods
    public final void finalMethod() {
        System.out.println("Final method");
    }
    
    // Methods with generic types
    public static <T> T identity(T value) {
        return value;
    }
    
    public static <T> List<T> createList(T... elements) {
        return Arrays.asList(elements);
    }
    
    public static <T extends Comparable<T>> T max(T a, T b) {
        return a.compareTo(b) > 0 ? a : b;
    }
    
    // Methods with varargs
    public static int sum(int... numbers) {
        int total = 0;
        for (int num : numbers) {
            total += num;
        }
        return total;
    }
    
    // Methods with arrays
    public static int[] reverse(int[] array) {
        int[] result = new int[array.length];
        for (int i = 0; i < array.length; i++) {
            result[i] = array[array.length - 1 - i];
        }
        return result;
    }
    
    // Methods with collections
    public static List<Integer> filterEven(List<Integer> numbers) {
        return numbers.stream()
                     .filter(n -> n % 2 == 0)
                     .collect(Collectors.toList());
    }
    
    // Methods with functional interfaces
    public static <T, R> List<R> map(List<T> list, Function<T, R> mapper) {
        return list.stream()
                   .map(mapper)
                   .collect(Collectors.toList());
    }
    
    public static <T> List<T> filter(List<T> list, Predicate<T> predicate) {
        return list.stream()
                   .filter(predicate)
                   .collect(Collectors.toList());
    }
    
    // Methods with lambda expressions
    public static void processNumbers(List<Integer> numbers) {
        numbers.forEach(n -> System.out.println("Number: " + n));
        
        List<Integer> doubled = numbers.stream()
                                      .map(n -> n * 2)
                                      .collect(Collectors.toList());
        
        Optional<Integer> max = numbers.stream()
                                      .max(Integer::compareTo);
    }
    
    // Methods with exception handling
    public static String readFile(String filename) throws IOException {
        if (filename == null || filename.isEmpty()) {
            throw new IllegalArgumentException("Filename cannot be null or empty");
        }
        // Simulate file reading
        return "File content: " + filename;
    }
    
    public static String safeReadFile(String filename) {
        try {
            return readFile(filename);
        } catch (IOException e) {
            System.err.println("Error reading file: " + e.getMessage());
            return null;
        }
    }
    
    // Methods with multiple exceptions
    public static void validateData(String data) throws IllegalArgumentException, IllegalStateException {
        if (data == null) {
            throw new IllegalArgumentException("Data cannot be null");
        }
        if (data.length() < 3) {
            throw new IllegalStateException("Data too short");
        }
    }
    
    // Recursive methods
    public static int factorial(int n) {
        if (n <= 1) {
            return 1;
        }
        return n * factorial(n - 1);
    }
    
    public static int fibonacci(int n) {
        if (n <= 1) {
            return n;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
    
    // Methods with annotations
    @Override
    public String toString() {
        return "BasicFunctions instance";
    }
    
    @Deprecated
    public static void oldMethod() {
        System.out.println("This method is deprecated");
    }
    
    @SuppressWarnings("unchecked")
    public static <T> T unsafeCast(Object obj) {
        return (T) obj;
    }
    
    // Async methods
    public static CompletableFuture<String> asyncGreet(String name) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                Thread.sleep(1000); // Simulate delay
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            return "Hello, " + name + "!";
        });
    }
    
    // Methods with builder pattern
    public static Builder builder() {
        return new Builder();
    }
    
    public static class Builder {
        private String name;
        private int age;
        
        public Builder name(String name) {
            this.name = name;
            return this;
        }
        
        public Builder age(int age) {
            this.age = age;
            return this;
        }
        
        public Person build() {
            return new Person(name, age);
        }
    }
    
    public static class Person {
        private final String name;
        private final int age;
        
        public Person(String name, int age) {
            this.name = name;
            this.age = age;
        }
        
        public String getName() {
            return name;
        }
        
        public int getAge() {
            return age;
        }
        
        @Override
        public String toString() {
            return String.format("Person{name='%s', age=%d}", name, age);
        }
    }
    
    // Methods with synchronized keyword
    public synchronized void synchronizedMethod() {
        System.out.println("Synchronized method");
    }
    
    public static synchronized void staticSynchronizedMethod() {
        System.out.println("Static synchronized method");
    }
    
    // Methods with native keyword (just declaration)
    public native void nativeMethod();
    
    // Methods with try-with-resources
    public static void processResource() {
        try (Scanner scanner = new Scanner(System.in)) {
            System.out.println("Processing input...");
            // Process input
        }
    }
    
    // Main method
    public static void main(String[] args) {
        System.out.println(greet("World"));
        System.out.println(add(5, 3));
        System.out.println(sum(1, 2, 3, 4, 5));
        
        List<Integer> numbers = Arrays.asList(1, 2, 3, 4, 5);
        List<Integer> evenNumbers = filterEven(numbers);
        System.out.println("Even numbers: " + evenNumbers);
        
        Person person = builder()
                .name("Alice")
                .age(30)
                .build();
        System.out.println(person);
        
        // Async example
        CompletableFuture<String> future = asyncGreet("Async World");
        future.thenAccept(System.out::println);
    }
}