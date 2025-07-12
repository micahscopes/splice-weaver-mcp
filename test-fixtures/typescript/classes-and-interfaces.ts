// Interfaces
interface User {
  id: number;
  name: string;
  email?: string;
  readonly createdAt: Date;
}

interface Admin extends User {
  permissions: string[];
  level: 'super' | 'regular';
}

// Generic interfaces
interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

// Function interfaces
interface Calculator {
  add(a: number, b: number): number;
  subtract(a: number, b: number): number;
}

// Classes
class Animal {
  protected name: string;
  private age: number;
  
  constructor(name: string, age: number) {
    this.name = name;
    this.age = age;
  }
  
  public speak(): string {
    return `${this.name} makes a sound`;
  }
  
  protected getAge(): number {
    return this.age;
  }
}

class Dog extends Animal {
  private breed: string;
  
  constructor(name: string, age: number, breed: string) {
    super(name, age);
    this.breed = breed;
  }
  
  public speak(): string {
    return `${this.name} barks`;
  }
  
  public getBreed(): string {
    return this.breed;
  }
}

// Abstract classes
abstract class Shape {
  abstract area(): number;
  abstract perimeter(): number;
  
  describe(): string {
    return `This shape has area ${this.area()} and perimeter ${this.perimeter()}`;
  }
}

class Circle extends Shape {
  constructor(private radius: number) {
    super();
  }
  
  area(): number {
    return Math.PI * this.radius ** 2;
  }
  
  perimeter(): number {
    return 2 * Math.PI * this.radius;
  }
}

// Generic classes
class Container<T> {
  private items: T[] = [];
  
  add(item: T): void {
    this.items.push(item);
  }
  
  get(index: number): T | undefined {
    return this.items[index];
  }
  
  size(): number {
    return this.items.length;
  }
}

// Static members
class MathUtils {
  static readonly PI = 3.14159;
  
  static add(a: number, b: number): number {
    return a + b;
  }
  
  static multiply(a: number, b: number): number {
    return a * b;
  }
}

// Getters and setters
class Temperature {
  private _celsius: number = 0;
  
  get celsius(): number {
    return this._celsius;
  }
  
  set celsius(value: number) {
    this._celsius = value;
  }
  
  get fahrenheit(): number {
    return this._celsius * 9/5 + 32;
  }
  
  set fahrenheit(value: number) {
    this._celsius = (value - 32) * 5/9;
  }
}

// Decorators (experimental)
function logged(target: any, propertyKey: string, descriptor: PropertyDescriptor) {
  const originalMethod = descriptor.value;
  descriptor.value = function(...args: any[]) {
    console.log(`Calling ${propertyKey} with args:`, args);
    return originalMethod.apply(this, args);
  };
}

class ApiService {
  @logged
  fetchUser(id: number): Promise<User> {
    return fetch(`/api/users/${id}`).then(res => res.json());
  }
}