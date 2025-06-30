#!/usr/bin/env python3
# Test file for ast-grep Python functionality

import os
import sys
import sqlite3
import logging
from typing import List, Dict, Optional

# This should trigger print statement warning
print("Starting Python ast-grep analysis test")

def main():
    """Main function to test various Python patterns"""
    # More print statements that should be detected
    print("Initializing application...")

    # Setup logging (good practice)
    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger(__name__)

    # This is the preferred way
    logger.info("Application initialized successfully")

    # Test database operations
    db_manager = DatabaseManager("test.db")

    # This should trigger SQL injection warning
    user_input = "'; DROP TABLE users; --"
    db_manager.execute_query(f"SELECT * FROM users WHERE name = '{user_input}'")

    # Test other functionality
    processor = DataProcessor()
    data = [1, 2, 3, 4, 5]
    result = processor.process_data(data)

    print(f"Processing result: {result}")  # Another print to detect

    return result

class DatabaseManager:
    """Database management class with potential security issues"""

    def __init__(self, db_path: str):
        self.db_path = db_path
        self.connection = None

    def connect(self):
        """Connect to database"""
        try:
            self.connection = sqlite3.connect(self.db_path)
            print("Database connected successfully")  # Print detection
        except sqlite3.Error as e:
            print(f"Database connection failed: {e}")  # Another print

    def execute_query(self, query: str):
        """Execute SQL query - potential injection point"""
        if not self.connection:
            self.connect()

        cursor = self.connection.cursor()
        # This should trigger SQL injection warning
        cursor.execute(query)
        return cursor.fetchall()

    def safe_query(self, user_id: int):
        """Example of safer query using parameters"""
        cursor = self.connection.cursor()
        cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))
        return cursor.fetchall()

    def close(self):
        """Close database connection"""
        if self.connection:
            self.connection.close()
            print("Database connection closed")  # More print usage

class DataProcessor:
    """Data processing class"""

    def __init__(self):
        self.data_cache: Dict[str, List[int]] = {}
        print("DataProcessor initialized")  # Print detection

    def process_data(self, data: List[int]) -> Dict[str, float]:
        """Process numerical data"""
        if not data:
            print("Warning: Empty data provided")  # Print detection
            return {}

        result = {
            'sum': sum(data),
            'average': sum(data) / len(data),
            'min': min(data),
            'max': max(data)
        }

        # Debug print that should be detected
        print(f"Processed data: {result}")

        return result

    def cache_data(self, key: str, data: List[int]):
        """Cache data for later use"""
        self.data_cache[key] = data
        print(f"Data cached with key: {key}")  # Another print

    def get_cached_data(self, key: str) -> Optional[List[int]]:
        """Retrieve cached data"""
        return self.data_cache.get(key)

def file_operations():
    """Function demonstrating file operations"""
    try:
        with open("test.txt", "r") as file:
            content = file.read()
            print(f"File content: {content}")  # Print detection
    except FileNotFoundError:
        print("File not found")  # Print detection
    except Exception as e:
        print(f"Error reading file: {e}")  # Print detection

def advanced_sql_operations():
    """Function with more SQL injection risks"""
    conn = sqlite3.connect(":memory:")
    cursor = conn.cursor()

    # Create test table
    cursor.execute("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)")

    # Dangerous SQL construction - should trigger warning
    user_name = "admin"
    query = f"INSERT INTO users VALUES (1, '{user_name}', 'admin@example.com')"
    cursor.execute(query)

    # Another dangerous pattern
    search_term = "test"
    cursor.execute(f"SELECT * FROM users WHERE name LIKE '%{search_term}%'")

    conn.close()

# Function with various patterns
def pattern_examples():
    """Function containing various patterns for detection"""

    # Multiple print statements
    print("Pattern analysis starting...")
    print("Checking different code patterns")
    print("Analysis in progress...")

    # List comprehension
    numbers = [x for x in range(10) if x % 2 == 0]
    print(f"Even numbers: {numbers}")

    # Dictionary comprehension
    squares = {x: x**2 for x in range(5)}
    print(f"Squares: {squares}")

    # Lambda function
    multiply = lambda x, y: x * y
    result = multiply(5, 3)
    print(f"Multiplication result: {result}")

    return numbers, squares, result

# Decorator example
def debug_decorator(func):
    """Decorator that prints function calls"""
    def wrapper(*args, **kwargs):
        print(f"Calling function: {func.__name__}")  # Print detection
        result = func(*args, **kwargs)
        print(f"Function {func.__name__} completed")  # Print detection
        return result
    return wrapper

@debug_decorator
def decorated_function(x: int, y: int) -> int:
    """Function with decorator"""
    return x + y

# Generator function
def number_generator(n: int):
    """Generator that yields numbers"""
    for i in range(n):
        print(f"Generating number: {i}")  # Print detection
        yield i

# Context manager
class PrintContext:
    """Context manager that prints entry and exit"""

    def __enter__(self):
        print("Entering context")  # Print detection
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print("Exiting context")  # Print detection

# Async function
async def async_operation():
    """Async function example"""
    print("Starting async operation")  # Print detection
    # Simulate async work
    import asyncio
    await asyncio.sleep(0.1)
    print("Async operation completed")  # Print detection
    return "Success"

if __name__ == "__main__":
    # Entry point with prints
    print("=== Python AST-Grep Analysis Test ===")

    main()
    file_operations()
    advanced_sql_operations()
    pattern_examples()

    # Test decorated function
    result = decorated_function(10, 20)
    print(f"Decorated function result: {result}")

    # Test generator
    gen = number_generator(3)
    for num in gen:
        print(f"Generated: {num}")

    # Test context manager
    with PrintContext() as ctx:
        print("Inside context manager")

    print("=== Analysis Complete ===")
