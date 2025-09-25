# demo_script.py
def old_function(data):
    results = []
    for i in data:
        if i % 2 == 0:
            results.append(str(i))
    return ",".join(results)
print(old_function(range(10)))
