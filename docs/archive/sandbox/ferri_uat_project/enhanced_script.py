```python
import datetime
import argparse

def get_current_time_formatted(format_string="%Y-%m-%d %H:%M:%S"):
    """Returns the current time formatted according to the given format string."""
    now = datetime.datetime.now()
    return now.strftime(format_string)

def write_timestamp_to_file(filename="timestamp.txt", format_string="%Y-%m-%d %H:%M:%S", append=True):
    """Writes the current timestamp to a file, optionally appending."""
    timestamp = get_current_time_formatted(format_string)
    mode = "a" if append else "w"
    try:
        with open(filename, mode) as f:
            f.write(timestamp + "\n")  # Add newline for readability
        print(f"Timestamp written to {filename}")
    except FileNotFoundError:
        print(f"Error: File {filename} not found.")
    except PermissionError:
        print(f"Error: Permission denied when writing to {filename}.")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Write timestamp to file.")
    parser.add_argument("filename", nargs="?", default="timestamp.txt", help="Filename to write to")
    parser.add_argument("-f", "--format", default="%Y-%m-%d %H:%M:%S", help="Timestamp format string")
    parser.add_argument("-o", "--overwrite", action="store_false", help="Overwrite file instead of appending") 
    args = parser.parse_args()
    write_timestamp_to_file(args.filename, args.format, not args.overwrite) 
```
