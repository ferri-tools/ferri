"""
A simple Flask application to demonstrate Ferri's code review capabilities.
This script contains intentional flaws for the demo.
"""
from flask import Flask, request
import os

app = Flask(__name__)

@app.route('/')
def index():
    name = request.args.get('name', 'World')
    return "Hello, " + name + "!"

# Flaw: This endpoint is vulnerable to command injection
@app.route('/files')
def list_files():
    directory = request.args.get('dir', '.')
    # This is dangerous!
    file_list = os.popen("ls " + directory).read()
    return f"<pre>{file_list}</pre>"

if __name__ == '__main__':
    app.run(debug=True)
