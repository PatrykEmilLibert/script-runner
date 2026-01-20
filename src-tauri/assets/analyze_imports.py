#!/usr/bin/env python3
"""
Python script dependency analyzer using AST parsing
This script is called from Rust backend to detect Python dependencies
"""

import ast
import sys
import json
from pathlib import Path

# Standard library modules
STDLIB_MODULES = {
    'sys', 'os', 'json', 're', 'math', 'random', 'datetime', 'time', 'collections',
    'itertools', 'functools', 'operator', 'string', 'io', 'pathlib', 'tempfile',
    'glob', 'fnmatch', 'linecache', 'shutil', 'gzip', 'bz2', 'zipfile', 'tarfile',
    'csv', 'configparser', 'hashlib', 'hmac', 'secrets', 'sqlite3', 'pickle',
    'copyreg', 'shelve', 'dbm', 'marshal', 'struct', 'difflib', 'textwrap',
    'unicodedata', 'stringprep', 'readline', 'rlcompleter', 'ast', 'symtable',
    'typing', 'pydoc', 'argparse', 'getopt', 'logging', 'getpass', 'curses',
    'platform', 'errno', 'ctypes', 'threading', 'multiprocessing', 'asyncio',
    'queue', 'socket', 'ssl', 'select', 'selectors', 'asynchat', 'asyncore',
    'signal', 'mmap', 'urllib', 'http', 'ftplib', 'poplib', 'imaplib', 'smtplib',
    'uuid', 'socketserver', 'xmlrpc', 'ipaddress', 'webbrowser', 'cgi', 'cgitb',
    'wsgiref', 'html', 'xml', 'traceback', 'gc', 'inspect', 'site', 'user',
    'distutils', 'venv', 'zipapp', 'unittest', 'doctest', 'pdb', 'profile',
    'pstats', 'trace', 'timeit', 'warnings', 'email', 'mailbox', 'mimetypes',
    'base64', 'binascii', 'quopri', 'uu', 'calendar', 'pprint', 'reprlib',
    'enum', 'numbers', 'cmath', 'decimal', 'fractions', 'statistics', 'array',
}

def extract_imports(file_path: str) -> set:
    """Extract all non-stdlib imports from a Python file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            tree = ast.parse(f.read())
    except Exception as e:
        return set()
    
    imports = set()
    
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                module_name = alias.name.split('.')[0]
                if module_name not in STDLIB_MODULES:
                    imports.add(module_name)
        elif isinstance(node, ast.ImportFrom):
            if node.module:
                module_name = node.module.split('.')[0]
                if module_name not in STDLIB_MODULES:
                    imports.add(module_name)
    
    return imports

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Please provide script path"}))
        sys.exit(1)
    
    script_path = sys.argv[1]
    imports = extract_imports(script_path)
    
    # Check for requirements.txt in same directory
    req_file = Path(script_path).parent / "requirements.txt"
    if req_file.exists():
        try:
            with open(req_file, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        package = line.split('==')[0].split('>=')[0].split('<=')[0].strip()
                        imports.add(package)
        except Exception:
            pass
    
    print(json.dumps({"packages": list(imports)}))

if __name__ == "__main__":
    main()
