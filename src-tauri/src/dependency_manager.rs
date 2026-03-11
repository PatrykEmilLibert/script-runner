use regex::Regex;
use std::collections::{hash_map::DefaultHasher, BTreeSet, HashMap, HashSet};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::script_encryption;
use crate::script_manager;

fn apply_no_console_window(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
    }
}

fn apply_macos_runtime_env(cmd: &mut Command) {
    #[cfg(target_os = "macos")]
    {
        // Force real macOS version reporting (avoid legacy 10.16 compatibility mode).
        cmd.env("SYSTEM_VERSION_COMPAT", "0");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = cmd;
    }
}

fn apply_python_process_isolation(cmd: &mut Command, python_exec: &PathBuf) {
    // Avoid inherited environment poisoning that can redirect stdlib resolution
    // to script folders and break ensurepip/pip bootstrapping.
    for key in [
        "PYTHONHOME",
        "PYTHONPATH",
        "PYTHONSTARTUP",
        "PYTHONUSERBASE",
        "PYTHONEXECUTABLE",
        "__PYVENV_LAUNCHER__",
    ] {
        cmd.env_remove(key);
    }

    // Ensure interpreter bootstrap runs in a stable location near python.exe,
    // not in a potentially hostile script directory containing pyvenv.cfg/Lib.
    if let Some(parent) = python_exec.parent() {
        cmd.current_dir(parent);
    }
}

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
    map.insert("docx", "python-docx");
    map.insert("pptx", "python-pptx");

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

    // Windows-specific modules provided by pywin32
    map.insert("pythoncom", "pywin32");
    map.insert("pywintypes", "pywin32");
    map.insert("win32api", "pywin32");
    map.insert("win32con", "pywin32");
    map.insert("win32gui", "pywin32");
    map.insert("win32process", "pywin32");
    map.insert("win32com", "pywin32");

    map
}

fn run_pip_install(
    python_exec: &PathBuf,
    packages: &[String],
    current_dir: Option<&PathBuf>,
) -> Result<std::process::Output, String> {
    ensure_pip_available(python_exec, current_dir)?;

    let mut cmd = Command::new(python_exec);
    cmd.args(["-m", "pip", "install"]);
    cmd.args(packages);
    apply_no_console_window(&mut cmd);
    apply_macos_runtime_env(&mut cmd);
    apply_python_process_isolation(&mut cmd, python_exec);

    if let Some(dir) = current_dir {
        #[cfg(not(target_os = "windows"))]
        {
            cmd.current_dir(dir);
        }

        #[cfg(target_os = "windows")]
        {
            let _ = dir;
        }
    }

    cmd.output()
        .map_err(|e| format!("Failed to run pip: {}", e))
}

fn run_python_command(
    python_exec: &PathBuf,
    args: &[&str],
    current_dir: Option<&PathBuf>,
) -> Result<std::process::Output, String> {
    let mut cmd = Command::new(python_exec);
    cmd.args(args);
    apply_no_console_window(&mut cmd);
    apply_macos_runtime_env(&mut cmd);
    apply_python_process_isolation(&mut cmd, python_exec);

    if let Some(dir) = current_dir {
        #[cfg(not(target_os = "windows"))]
        {
            cmd.current_dir(dir);
        }

        #[cfg(target_os = "windows")]
        {
            let _ = dir;
        }
    }

    cmd.output()
        .map_err(|e| format!("Failed to run Python command: {}", e))
}

fn is_pip_available(python_exec: &PathBuf, current_dir: Option<&PathBuf>) -> Result<bool, String> {
    let output = run_python_command(python_exec, &["-m", "pip", "--version"], current_dir)?;
    Ok(output.status.success())
}

fn ensure_pip_available(
    python_exec: &PathBuf,
    current_dir: Option<&PathBuf>,
) -> Result<(), String> {
    if is_pip_available(python_exec, current_dir)? {
        return Ok(());
    }

    log::warn!(
        "pip is missing for Python runtime {}, attempting bootstrap",
        python_exec.display()
    );

    let ensurepip =
        run_python_command(python_exec, &["-m", "ensurepip", "--upgrade"], current_dir)?;
    if ensurepip.status.success() && is_pip_available(python_exec, current_dir)? {
        log::info!(
            "Bootstrapped pip using ensurepip for {}",
            python_exec.display()
        );
        return Ok(());
    }

    let bootstrap_code = r#"import pathlib, tempfile, urllib.request, subprocess, sys
url = 'https://bootstrap.pypa.io/get-pip.py'
dest = pathlib.Path(tempfile.gettempdir()) / 'get-pip.py'
urllib.request.urlretrieve(url, dest)
subprocess.check_call([sys.executable, str(dest), '--no-warn-script-location'])"#;

    let get_pip = run_python_command(python_exec, &["-c", bootstrap_code], current_dir)?;
    if get_pip.status.success() && is_pip_available(python_exec, current_dir)? {
        log::info!(
            "Bootstrapped pip using get-pip.py for {}",
            python_exec.display()
        );
        return Ok(());
    }

    let ensurepip_stderr = String::from_utf8_lossy(&ensurepip.stderr);
    let get_pip_stderr = String::from_utf8_lossy(&get_pip.stderr);

    Err(format!(
        "pip is not available for Python runtime '{}'. ensurepip failed with: {}\nget-pip bootstrap failed with: {}",
        python_exec.display(),
        ensurepip_stderr.trim(),
        get_pip_stderr.trim()
    ))
}

fn extract_unavailable_packages(stderr: &str) -> HashSet<String> {
    let mut unavailable = HashSet::new();

    let could_not_find =
        Regex::new(r"Could not find a version that satisfies the requirement\s+([^\s\(]+)")
            .unwrap();
    for caps in could_not_find.captures_iter(stderr) {
        if let Some(pkg) = caps.get(1) {
            unavailable.insert(pkg.as_str().trim().to_string());
        }
    }

    let no_matching = Regex::new(r"No matching distribution found for\s+([^\s]+)").unwrap();
    for caps in no_matching.captures_iter(stderr) {
        if let Some(pkg) = caps.get(1) {
            unavailable.insert(pkg.as_str().trim().to_string());
        }
    }

    unavailable
}

fn install_packages_resilient(
    python_exec: &PathBuf,
    packages: &[String],
    current_dir: Option<&PathBuf>,
) -> Result<(), String> {
    if packages.is_empty() {
        return Ok(());
    }

    let output = run_pip_install(python_exec, packages, current_dir)?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let unavailable = extract_unavailable_packages(&stderr);

    if unavailable.is_empty() {
        return Err(format!("{}\n{}", stdout, stderr));
    }

    let filtered: Vec<String> = packages
        .iter()
        .filter(|pkg| !unavailable.contains(pkg.as_str()))
        .cloned()
        .collect();

    if filtered.is_empty() {
        return Err(format!("{}\n{}", stdout, stderr));
    }

    let retry_output = run_pip_install(python_exec, &filtered, current_dir)?;
    if retry_output.status.success() {
        log::warn!(
            "Skipped non-installable packages: {:?}. Installed remaining packages: {:?}",
            unavailable,
            filtered
        );
        return Ok(());
    }

    let retry_stderr = String::from_utf8_lossy(&retry_output.stderr);
    let retry_stdout = String::from_utf8_lossy(&retry_output.stdout);
    Err(format!(
        "Skipped non-installable packages: {:?}\n\nRetry with remaining packages failed:\n{}\n{}",
        unavailable, retry_stdout, retry_stderr
    ))
}

const STDLIB_MODULES: &[&str] = &[
    "sys",
    "os",
    "base64",
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
    "dataclasses",
    "enum",
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
    "concurrent",
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
        .split(['=', '>', '<', '!', '[', ' '])
        .next()
        .unwrap_or(module)
        .trim();

    STDLIB_MODULES.contains(&package)
}

fn force_install_all_requirements() -> bool {
    env::var("SR_FORCE_INSTALL_ALL_REQUIREMENTS")
        .map(|v| v != "0")
        .unwrap_or(true)
}

fn normalize_requirement_entry(line: &str) -> Option<String> {
    let raw = line.trim();
    if raw.is_empty() || raw.starts_with('#') || raw.starts_with('-') {
        return None;
    }

    let package = raw
        .split(['=', '>', '<', '!', '[', ' '])
        .next()
        .unwrap_or(raw)
        .trim();

    if package.is_empty() {
        return None;
    }

    // Skip pseudo-modules and invalid names that pip cannot install (e.g. __future__).
    if package.starts_with('_') || (package.starts_with("__") && package.ends_with("__")) {
        return None;
    }

    Some(package.to_string())
}

fn requirements_to_packages(content: &str, include_stdlib: bool) -> Vec<String> {
    let import_map = get_import_to_package_map();
    let mut skipped_stdlib: BTreeSet<String> = BTreeSet::new();

    let packages: Vec<String> = content
        .lines()
        .filter_map(normalize_requirement_entry)
        .filter(|package| {
            if is_stdlib(package) {
                skipped_stdlib.insert(package.to_string());
                return false;
            }
            true
        })
        .map(|package| {
            // Map import/module name to pip package name if needed
            let mapped = import_map
                .get(package.as_str())
                .copied()
                .unwrap_or(package.as_str());
            mapped.to_string()
        })
        .collect();

    if include_stdlib && !skipped_stdlib.is_empty() {
        log::warn!(
            "Ignoring stdlib entries in requirements.txt (not installable via pip): {:?}",
            skipped_stdlib
        );
    }

    packages
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
                if let Some(package) = normalize_requirement_entry(line) {
                    imports.insert(package);
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

    let include_stdlib = force_install_all_requirements();
    if include_stdlib {
        log::info!(
            "SR_FORCE_INSTALL_ALL_REQUIREMENTS active: installing all entries from requirements.txt"
        );
    }
    let packages = requirements_to_packages(&content, include_stdlib);

    if packages.is_empty() {
        log::info!(
            "No packages to install from requirements in {:?}",
            script_dir
        );
        return Ok(());
    }

    log::info!("Installing packages: {:?}", packages);

    if let Err(raw_error) = install_packages_resilient(python_exec, &packages, Some(script_dir)) {
        let stderr = raw_error.as_str();
        let stdout = "";

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
    let official_root = scripts_root.join("official");
    let user_root = script_manager::get_user_scripts_root(scripts_root.as_path());

    // Find every folder containing main.py (handles scripts/ and official/)
    for entry in WalkDir::new(scripts_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let dir = entry.path();

        let is_official = dir.starts_with(&official_root);
        let is_current_user_script = dir.starts_with(&user_root);
        if !is_official && !is_current_user_script {
            continue;
        }

        let main_py = dir.join("main.py");
        let main_enc = dir.join("main.py.enc");
        if !main_py.exists() && !main_enc.exists() {
            continue;
        }

        let req_path = dir.join("requirements.txt");

        if req_path.exists() {
            match fs::read_to_string(&req_path) {
                Ok(content) => {
                    let include_stdlib = force_install_all_requirements();
                    if include_stdlib {
                        log::info!(
                            "SR_FORCE_INSTALL_ALL_REQUIREMENTS active: aggregating all requirements for {}",
                            dir.display()
                        );
                    }
                    for package in requirements_to_packages(&content, include_stdlib) {
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
            let detected = if main_py.exists() {
                detect_dependencies(&main_py).await
            } else {
                let temp_analysis =
                    std::env::temp_dir().join(format!("sr_deps_{}.py", uuid::Uuid::new_v4()));

                let deps_result = match script_encryption::decrypt_script(&main_enc) {
                    Ok(content) => {
                        if let Err(e) = fs::write(&temp_analysis, content) {
                            Err(format!("failed to write temp decrypted script: {}", e))
                        } else {
                            detect_dependencies(&temp_analysis).await
                        }
                    }
                    Err(e) => Err(format!(
                        "failed to decrypt script for dependency detection: {}",
                        e
                    )),
                };

                let _ = fs::remove_file(&temp_analysis);
                deps_result
            };

            match detected {
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

    if let Err(raw_error) = install_packages_resilient(python_exec, &combined, None) {
        let stderr = raw_error.as_str();
        let stdout = "";

        let mut error_msg = format!("❌ Failed to install {} packages\n", combined.len());

        // Enhanced error diagnostics
        if stderr.contains("Could not find a version")
            || stderr.contains("No matching distribution")
        {
            error_msg.push_str("\n💡 Some packages don't exist or have wrong names\n");
        }
        if stderr.contains("externally-managed-environment") {
            error_msg.push_str(
                "\n⚠️ System Python is protected - using virtual environment recommended\n",
            );
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

    let import_map = get_import_to_package_map();
    let mut skipped_stdlib: BTreeSet<String> = BTreeSet::new();

    // Map package names in case they're import names instead of pip names
    let mapped_packages: Vec<String> = packages
        .iter()
        .filter_map(|pkg| normalize_requirement_entry(pkg))
        .filter(|package| {
            if is_stdlib(package) {
                skipped_stdlib.insert(package.to_string());
                return false;
            }
            true
        })
        .map(|package| {
            let package_str = package.as_str();

            let mapped = import_map.get(package_str).copied().unwrap_or(package_str);
            mapped.to_string()
        })
        .collect();

    if !skipped_stdlib.is_empty() {
        log::warn!(
            "Skipping stdlib modules during dependency install: {:?}",
            skipped_stdlib
        );
    }

    if mapped_packages.is_empty() {
        return Ok(());
    }

    if let Err(raw_error) = install_packages_resilient(python_exec, &mapped_packages, None) {
        return Err(format!("Pip install failed: {}", raw_error));
    }

    Ok(())
}
