use regex::Regex;
use std::collections::{hash_map::DefaultHasher, BTreeSet, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

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

    let output = Command::new(python_exec)
        .args(&["-m", "pip", "install", "-r", "requirements.txt"])
        .current_dir(script_dir)
        .output()
        .map_err(|e| format!("Failed to run pip: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pip install failed: {}", stderr));
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
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }
                        aggregated.insert(trimmed.to_string());
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
        return Err(format!("Pip install failed: {}", stderr));
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
