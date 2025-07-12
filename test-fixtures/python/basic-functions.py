#!/usr/bin/env python3

# Basic function definitions
def greet(name):
    return f"Hello, {name}!"

def add(a, b):
    """Add two numbers together."""
    return a + b

# Functions with default parameters
def greet_user(name, greeting="Hello"):
    return f"{greeting}, {name}!"

# Functions with keyword arguments
def create_person(name, age=None, email=None):
    person = {"name": name}
    if age is not None:
        person["age"] = age
    if email is not None:
        person["email"] = email
    return person

# Functions with *args and **kwargs
def sum_all(*args):
    return sum(args)

def print_info(**kwargs):
    for key, value in kwargs.items():
        print(f"{key}: {value}")

def flexible_function(*args, **kwargs):
    print(f"Args: {args}")
    print(f"Kwargs: {kwargs}")

# Lambda functions
square = lambda x: x ** 2
add_ten = lambda x: x + 10
sort_by_length = lambda items: sorted(items, key=len)

# Functions with annotations
def calculate_area(width: float, height: float) -> float:
    """Calculate the area of a rectangle."""
    return width * height

def process_data(data: list[int]) -> list[int]:
    """Process a list of integers."""
    return [x * 2 for x in data if x > 0]

# Generator functions
def count_up_to(n):
    """Generator that yields numbers from 0 to n."""
    i = 0
    while i < n:
        yield i
        i += 1

def fibonacci():
    """Generator for Fibonacci sequence."""
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b

# Decorator functions
def timer(func):
    """Decorator to time function execution."""
    import time
    def wrapper(*args, **kwargs):
        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        print(f"{func.__name__} took {end - start:.4f} seconds")
        return result
    return wrapper

def validate_types(*types):
    """Decorator to validate argument types."""
    def decorator(func):
        def wrapper(*args, **kwargs):
            for i, (arg, expected_type) in enumerate(zip(args, types)):
                if not isinstance(arg, expected_type):
                    raise TypeError(f"Argument {i} must be {expected_type.__name__}")
            return func(*args, **kwargs)
        return wrapper
    return decorator

# Class methods
class Calculator:
    def __init__(self, initial_value=0):
        self.value = initial_value
    
    def add(self, x):
        """Add x to the current value."""
        self.value += x
        return self
    
    def multiply(self, x):
        """Multiply current value by x."""
        self.value *= x
        return self
    
    @property
    def result(self):
        """Get the current result."""
        return self.value
    
    @staticmethod
    def absolute_add(a, b):
        """Add two numbers and return absolute value."""
        return abs(a + b)
    
    @classmethod
    def from_string(cls, value_str):
        """Create calculator from string value."""
        return cls(int(value_str))

# Async functions
import asyncio

async def fetch_data(url):
    """Fetch data from URL asynchronously."""
    # Simulate async operation
    await asyncio.sleep(1)
    return f"Data from {url}"

async def process_multiple_urls(urls):
    """Process multiple URLs concurrently."""
    tasks = [fetch_data(url) for url in urls]
    return await asyncio.gather(*tasks)

# Context manager functions
from contextlib import contextmanager

@contextmanager
def temporary_value(obj, attr, temp_value):
    """Temporarily set an attribute value."""
    old_value = getattr(obj, attr)
    setattr(obj, attr, temp_value)
    try:
        yield
    finally:
        setattr(obj, attr, old_value)

# Functions with error handling
def safe_divide(a, b):
    """Safely divide two numbers."""
    try:
        return a / b
    except ZeroDivisionError:
        print("Cannot divide by zero!")
        return None
    except TypeError:
        print("Invalid types for division!")
        return None

def validate_email(email):
    """Validate email format."""
    import re
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    if re.match(pattern, email):
        return True
    else:
        raise ValueError(f"Invalid email format: {email}")

# Recursive functions
def factorial(n):
    """Calculate factorial recursively."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def binary_search(arr, target, low=0, high=None):
    """Binary search implementation."""
    if high is None:
        high = len(arr) - 1
    
    if low > high:
        return -1
    
    mid = (low + high) // 2
    if arr[mid] == target:
        return mid
    elif arr[mid] > target:
        return binary_search(arr, target, low, mid - 1)
    else:
        return binary_search(arr, target, mid + 1, high)

# Functions with unpacking
def process_coordinates(point):
    """Process coordinate tuple."""
    x, y = point
    return x + y

def merge_dicts(dict1, dict2):
    """Merge two dictionaries."""
    return {**dict1, **dict2}

# Main execution
if __name__ == "__main__":
    # Test function calls
    print(greet("World"))
    print(add(5, 3))
    print(create_person("Alice", age=30, email="alice@example.com"))