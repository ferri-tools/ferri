def fibonacci(n):
    """
    Calculates the nth Fibonacci number using dynamic programming.
    """
    if n <= 1:
        return n
    fib = [0] * (n + 1)
    fib[1] = 1
    for i in range(2, n + 1):
        fib[i] = fib[i - 1] + fib[i - 2]
    return fib[n]

if __name__ == "__main__":
    n = 10  # Example usage
    result = fibonacci(n)
    print(f"The {n}th Fibonacci number is: {result}")
