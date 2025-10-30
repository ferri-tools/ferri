def calculate_error(actual, predicted):
  """Calculates the absolute error."""
  return abs(actual - predicted)

if __name__ == "__main__":
  actual_value = float(input("Enter the actual value: "))
  predicted_value = float(input("Enter the predicted value: "))
  error = calculate_error(actual_value, predicted_value)
  print(f"The error is: {error}")
