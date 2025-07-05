#!/usr/bin/env python3
"""
Simple calculator with basic arithmetic operations.
Demonstrates Python functions and error handling.
"""

def add(x, y):
    """Add two numbers."""
    return x + y

def subtract(x, y):
    """Subtract two numbers."""
    return x - y

def multiply(x, y):
    """Multiply two numbers."""
    return x * y

def divide(x, y):
    """Divide two numbers with error handling."""
    if y == 0:
        raise ValueError("Cannot divide by zero")
    return x / y

if __name__ == "__main__":
    print("Calculator module loaded")
    result = add(10, 5)
    print(f"10 + 5 = {result}")