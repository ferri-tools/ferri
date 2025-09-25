# pm/demo_script.py
import os

def old_and_inefficient_function(data):
    # This is a sample script with obvious room for improvement.
    results = []
    for i in data:
        if i % 2 == 0: # Inefficient check
            results.append(str(i))
    return ",".join(results)

def another_function():
    # This function is unused.
    pass

if __name__ == "__main__":
    my_data = range(20)
    print(old_and_inefficient_function(my_data))
