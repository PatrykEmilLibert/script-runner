use regex::Regex;
use std::collections::{hash_map::DefaultHasher, BTreeSet, HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

// Map import names to pip package names for packages with mismatched names
// Based on common PyPI package name mismatches
fn get_import_to_package_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    
    // Machine Learning & Data Science
    map.insert("sklearn", "scikit-learn");
    map.insert("cv2", "opencv-python");
    map.insert("cv", "opencv-python");
    map.insert("torch", "torch");
    map.insert("tensorflow", "tensorflow");
    map.insert("keras", "keras");
    map.insert("xgboost", "xgboost");
    
    // Image & Media
    map.insert("PIL", "Pillow");
    map.insert("Image", "Pillow");
    
    // Configuration & Data Formats
    map.insert("yaml", "pyyaml");
    map.insert("tomli", "tomli");
    
    // Web Scraping & Parsing
    map.insert("bs4", "beautifulsoup4");
    map.insert("lxml", "lxml");
    map.insert("html5lib", "html5lib");
    
    // Database Drivers
    map.insert("MySQLdb", "mysqlclient");
    map.insert("psycopg2", "psycopg2-binary");
    map.insert("psycopg", "psycopg");
    map.insert("sqlite3", "sqlite3");
    map.insert("pymongo", "pymongo");
    map.insert("redis", "redis");
    
    // Environment & Configuration
    map.insert("dotenv", "python-dotenv");
    map.insert("dateutil", "python-dateutil");
    map.insert("serial", "pyserial");
    map.insert("configparser", "configparser");
    
    // HTTP & Network
    map.insert("requests", "requests");
    map.insert("urllib3", "urllib3");
    map.insert("httpx", "httpx");
    
    // Async & Async HTTP
    map.insert("aiohttp", "aiohttp");
    map.insert("httplib2", "httplib2");
    
    // Testing
    map.insert("pytest", "pytest");
    map.insert("mock", "mock");
    
    // Text Processing & NLP
    map.insert("nltk", "nltk");
    map.insert("spacy", "spacy");
    map.insert("textblob", "textblob");
    
    // Cryptography
    map.insert("Crypto", "pycryptodome");
    map.insert("cryptography", "cryptography");
    
    // Data Format Parsing
    map.insert("openpyxl", "openpyxl");
    map.insert("xlrd", "xlrd");
    map.insert("xlwt", "xlwt");
    
    // Scientific Computing
    map.insert("scipy", "scipy");
    map.insert("sympy", "sympy");
    map.insert("pandas", "pandas");
    map.insert("numpy", "numpy");
    
    // Visualization
    map.insert("matplotlib", "matplotlib");
    map.insert("plotly", "plotly");
    map.insert("seaborn", "seaborn");
    map.insert("bokeh", "bokeh");
    
    // PDF Processing
    map.insert("PyPDF2", "PyPDF2");
    map.insert("pdfplumber", "pdfplumber");
    map.insert("reportlab", "reportlab");
    
    // Command Line & Terminal
    map.insert("click", "click");
    map.insert("typer", "typer");
    map.insert("rich", "rich");
    map.insert("colorama", "colorama");
    
    // Date & Time
    map.insert("arrow", "arrow");
    map.insert("pendulum", "pendulum");
    map.insert("pytz", "pytz");
    
    // File & Path Processing
    map.insert("pathlib2", "pathlib2");
    map.insert("watchdog", "watchdog");
    
    // Compression & Archives
    map.insert("tarfile", "tarfile");
    map.insert("zipfile", "zipfile");
    map.insert("gzip", "gzip");
    map.insert("rarfile", "rarfile");
    
    // JSON & Serialization
    map.insert("ujson", "ujson");
    map.insert("orjson", "orjson");
    map.insert("msgpack", "msgpack");
    
    // API & Web Frameworks
    map.insert("flask", "flask");
    map.insert("django", "django");
    map.insert("fastapi", "fastapi");
    map.insert("starlette", "starlette");
    map.insert("bottle", "bottle");
    map.insert("tornado", "tornado");
    
    // Utilities
    map.insert("pydantic", "pydantic");
    map.insert("attrs", "attrs");
    map.insert("six", "six");
    map.insert("future", "future");
    
    map
}

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
    "tkinter",
    "turtle",
    "graphlib",
    "code",
    "codeop",
    "cmd",
    "shlex",
    "ensurepip",
    "lib2to3",
    "builtins",
    "encodings",
    "codecs",
    "locale",
    "gettext",
    "__future__",
];

fn is_stdlib(module: &str) -> bool {
    // Extract package name (before any version specifier or extras)
    let package = module
        .split(|c: char| c == '=' || c == '>' || c == '<' || c == '!' || c == '[' || c == ' ')
        .next()
        .unwrap_or(module)
        .trim();
    
    STDLIB_MODULES.contains(&package)
}

fn filter_stdlib_from_requirements(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| !is_stdlib(line))
        .map(|s| s.to_string())
        .collect()
}

pub async fn detect_dependencies(script_path: &PathBuf) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(script_path)
        .map_err(|e| format!("Failed to read script: {}", e))?;

    let mut imports = HashSet::new();
    let import_map = get_import_to_package_map();

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
                // Map import name to package name if needed
                let package_name = import_map.get(root_module).copied().unwrap_or(root_module);
                imports.insert(package_name.to_string());
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

fn hash_file(path: &Path) -> Result<u64, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    Ok(hasher.finish())
}

pub async fn ensure_requirements(
    script_dir: &PathBuf,
    python_exec: &PathBuf,
) -> Result<(), String> {
    let req_path = script_dir.join("requirements.txt");
    if !req_path.exists() {
        return Ok(());
    }

    let cache_path = script_dir.join(".deps-installed");
    let current_hash = hash_file(&req_path)?;

    // Skip install if hash matches cached
    if cache_path.exists() {
        if let Ok(cache_str) = fs::read_to_string(&cache_path) {
            if let Ok(saved_hash) = cache_str.trim().parse::<u64>() {
                if saved_hash == current_hash {
                    return Ok(());
                }
            }
        }
    }

    // Read and filter stdlib modules from requirements
    let content = fs::read_to_string(&req_path)
        .map_err(|e| format!("Failed to read requirements.txt: {}", e))?;
    
    let packages = filter_stdlib_from_requirements(&content);
    
    if packages.is_empty() {
        log::info!("No non-stdlib packages to install in {:?}", script_dir);
        return Ok(());
    }

    log::info!("Installing packages: {:?}", packages);

    let output = Command::new(python_exec)
        .args(&["-m", "pip", "install"])
        .args(&packages)
        .current_dir(script_dir)
        .output()
        .map_err(|e| format!("Failed to run pip: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse pip error for better diagnostics
        let mut error_msg = format!("❌ Failed to install packages: {}\n", packages.join(", "));
        
        if stderr.contains("Could not find a version") {
            error_msg.push_str("\n💡 Suggestion: Package version not found. Try:\n");
            error_msg.push_str("  • Check package name spelling\n");
            error_msg.push_str("  • Remove version constraints from requirements.txt\n");
        } else if stderr.contains("No matching distribution") {
            error_msg.push_str("\n💡 Suggestion: Package doesn't exist. Try:\n");
            error_msg.push_str("  • Verify package name on PyPI.org\n");
            error_msg.push_str("  • Check for typos in requirements.txt\n");
        } else if stderr.contains("THESE PACKAGES DO NOT MATCH THE HASHES") {
            error_msg.push_str("\n💡 Suggestion: Hash mismatch. Try:\n");
            error_msg.push_str("  • Remove hash constraints from requirements.txt\n");
            error_msg.push_str("  • Update pip: python -m pip install --upgrade pip\n");
        } else if stderr.contains("externally-managed-environment") {
            error_msg.push_str("\n💡 Suggestion: System Python is protected. Try:\n");
            error_msg.push_str("  • Use a virtual environment (venv)\n");
            error_msg.push_str("  • Or add --break-system-packages flag\n");
        } else if stderr.contains("SSLError") || stderr.contains("certificate") {
            error_msg.push_str("\n💡 Suggestion: SSL/Certificate error. Try:\n");
            error_msg.push_str("  • Check internet connection\n");
            error_msg.push_str("  • Update CA certificates\n");
            error_msg.push_str("  • Use --trusted-host flag\n");
        }
        
        error_msg.push_str(&format!("\n📋 Full error:\n{}\n{}", stdout, stderr));
        
        return Err(error_msg);
    }

    // Cache hash to avoid reinstalling unchanged deps
    if let Err(e) = fs::write(&cache_path, current_hash.to_string()) {
        eprintln!("Failed to write deps cache: {}", e);
    }

    Ok(())
}

pub async fn ensure_all_scripts_requirements(
    scripts_root: &PathBuf,
    python_exec: &PathBuf,
) -> Result<(), String> {
    if !scripts_root.exists() {
        return Ok(());
    }

    let mut errors: Vec<String> = Vec::new();
    let mut aggregated: BTreeSet<String> = BTreeSet::new();

    // Find every folder containing main.py (handles scripts/ and official/)
    for entry in WalkDir::new(scripts_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let dir = entry.path();
        let main_py = dir.join("main.py");
        if !main_py.exists() {
            continue;
        }

        let req_path = dir.join("requirements.txt");

        if req_path.exists() {
            match fs::read_to_string(&req_path) {
                Ok(content) => {
                    // Filter out stdlib modules from requirements
                    for package in filter_stdlib_from_requirements(&content) {
                        aggregated.insert(package);
                    }
                }
                Err(e) => errors.push(format!(
                    "{}: failed to read requirements: {}",
                    dir.display(),
                    e
                )),
            }
        } else {
            match detect_dependencies(&main_py).await {
                Ok(deps) => {
                    for dep in &deps {
                        aggregated.insert(dep.to_string());
                    }
                    if !deps.is_empty() {
                        if let Err(e) = fs::write(&req_path, deps.join("\n")) {
                            errors.push(format!(
                                "{}: failed to persist generated requirements: {}",
                                dir.display(),
                                e
                            ));
                        }
                    }
                }
                Err(e) => errors.push(format!("{}: failed to detect deps: {}", dir.display(), e)),
            }
        }
    }

    // Nothing to install
    if aggregated.is_empty() {
        return if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Dependency install issues:\n{}", errors.join("\n")))
        };
    }

    let combined: Vec<String> = aggregated.into_iter().collect();
    let mut hasher = DefaultHasher::new();
    combined.hash(&mut hasher);
    let combined_hash = hasher.finish();
    let cache_path = scripts_root.join(".deps-installed-all");

    if cache_path.exists() {
        if let Ok(cache_str) = fs::read_to_string(&cache_path) {
            if let Ok(saved_hash) = cache_str.trim().parse::<u64>() {
                if saved_hash == combined_hash {
                    return if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(format!("Dependency install issues:\n{}", errors.join("\n")))
                    };
                }
            }
        }
    }

    let output = Command::new(python_exec)
        .args(&["-m", "pip", "install"])
        .args(&combined)
        .output()
        .map_err(|e| format!("Failed to run pip: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let mut error_msg = format!("❌ Failed to install {} packages\n", combined.len());
        
        // Enhanced error diagnostics
        if stderr.contains("Could not find a version") || stderr.contains("No matching distribution") {
            error_msg.push_str("\n💡 Some packages don't exist or have wrong names\n");
        }
        if stderr.contains("externally-managed-environment") {
            error_msg.push_str("\n⚠️ System Python is protected - using virtual environment recommended\n");
        }
        
        error_msg.push_str(&format!("\n📋 Details:\n{}\n{}", stdout, stderr));
        
        return Err(error_msg);
    }

    if let Err(e) = fs::write(&cache_path, combined_hash.to_string()) {
        eprintln!("Failed to write global deps cache: {}", e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Dependency install issues (install succeeded but some scripts reported issues):\n{}",
            errors.join("\n")
        ))
    }
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
