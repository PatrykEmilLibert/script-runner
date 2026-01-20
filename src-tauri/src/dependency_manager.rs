use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

const STDLIB_MODULES: &[&str] = &[
    "sys",
    "os",
    "json",
    "re",
    "math",
    "random",
    "datetime",
    "time",
    "collections",
    "itertools",
    "functools",
    "operator",
    "string",
    "io",
    "pathlib",
    "tempfile",
    "glob",
    "fnmatch",
    "linecache",
    "shutil",
    "gzip",
    "bz2",
    "zipfile",
    "tarfile",
    "csv",
    "configparser",
    "hashlib",
    "hmac",
    "secrets",
    "sqlite3",
    "pickle",
    "copyreg",
    "shelve",
    "dbm",
    "marshal",
    "struct",
    "difflib",
    "textwrap",
    "unicodedata",
    "stringprep",
    "readline",
    "rlcompleter",
    "ast",
    "symtable",
    "typing",
    "pydoc",
    "argparse",
    "getopt",
    "logging",
    "getpass",
    "curses",
    "platform",
    "errno",
    "ctypes",
    "threading",
    "multiprocessing",
    "asyncio",
    "queue",
    "socket",
    "ssl",
    "select",
    "selectors",
    "asynchat",
    "asyncore",
    "signal",
    "mmap",
    "urllib",
    "http",
    "ftplib",
    "poplib",
    "imaplib",
    "smtplib",
    "uuid",
    "socketserver",
    "xmlrpc",
    "ipaddress",
    "webbrowser",
    "cgi",
    "cgitb",
    "wsgiref",
    "html",
    "xml",
    "traceback",
    "gc",
    "inspect",
    "site",
    "user",
    "distutils",
    "venv",
    "zipapp",
    "unittest",
    "doctest",
    "pdb",
    "profile",
    "pstats",
    "trace",
    "timeit",
    "warnings",
    "email",
    "mailbox",
    "mimetypes",
];

pub async fn detect_dependencies(script_path: &PathBuf) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(script_path)
        .map_err(|e| format!("Failed to read script: {}", e))?;

    let mut imports = HashSet::new();

    // Regex for: import X, import X as Y, from X import Y
    let import_re = Regex::new(r"^(?:from\s+(\S+)|import\s+(\S+))").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#") || trimmed.is_empty() {
            continue;
        }

        if let Some(caps) = import_re.captures(trimmed) {
            let module = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");

            let root_module = module.split('.').next().unwrap_or("");

            // Filter out stdlib modules
            if !STDLIB_MODULES.contains(&root_module) && !root_module.is_empty() {
                imports.insert(root_module.to_string());
            }
        }
    }

    // Check for existing requirements.txt
    let req_file = script_path.parent().unwrap().join("requirements.txt");
    if req_file.exists() {
        if let Ok(req_content) = std::fs::read_to_string(&req_file) {
            for line in req_content.lines() {
                let package = line.split('=').next().unwrap_or("").trim();
                if !package.is_empty() {
                    imports.insert(package.to_string());
                }
            }
        }
    }

    Ok(imports.into_iter().collect())
}

pub async fn install_dependencies(
    packages: &[String],
    python_exec: &PathBuf,
) -> Result<(), String> {
    if packages.is_empty() {
        return Ok(());
    }

    let output = Command::new(python_exec)
        .args(&["-m", "pip", "install"])
        .args(packages)
        .output()
        .map_err(|e| format!("Failed to run pip: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pip install failed: {}", stderr));
    }

    Ok(())
}
