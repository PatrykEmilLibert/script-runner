use git2::Repository;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::script_manager;

fn count_scripts(scripts_dir: &PathBuf) -> usize {
    if !scripts_dir.exists() {
        return 0;
    }

    let official_root = scripts_dir.join("official");
    let user_root = script_manager::get_user_scripts_root(scripts_dir.as_path());
    let legacy_user_root = script_manager::get_legacy_user_scripts_root(scripts_dir.as_path());

    WalkDir::new(scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| {
            let path = e.path();
            let is_official = path.starts_with(&official_root);
            let is_current_user_script = path.starts_with(&user_root);
            let is_legacy_user_script = path.starts_with(&legacy_user_root);

            (is_official || is_current_user_script || is_legacy_user_script)
                && (path.join("main.py").exists() || path.join("main.py.enc").exists())
        })
        .count()
}

pub fn sync_scripts(scripts_dir: &PathBuf) -> Result<String, String> {
    log::info!("Syncing scripts to: {:?}", scripts_dir);

    // Count before sync
    let count_before = count_scripts(scripts_dir);

    if !scripts_dir.exists() {
        log::info!("Creating scripts directory: {:?}", scripts_dir);
        std::fs::create_dir_all(scripts_dir)
            .map_err(|e| format!("Failed to create scripts dir: {}", e))?;
    }

    // Try to open repo at scripts_dir; if missing, clone from env or default URL
    let result = match Repository::open(scripts_dir) {
        Ok(_) => {
            log::info!("Existing repository found, syncing...");
            sync_from_git(scripts_dir)
        }
        Err(e) => {
            log::info!("No repository found ({}), will clone", e);
            let remote_url = std::env::var("SCRIPTS_REPO_URL").unwrap_or_else(|_| {
                "https://github.com/PatrykEmilLibert/script-runner-scripts.git".to_string()
            });
            log::info!("Cloning scripts repo from: {}", remote_url);
            log::info!("Target directory: {:?}", scripts_dir);

            match Repository::clone(&remote_url, scripts_dir) {
                Ok(_) => {
                    log::info!("Repository cloned successfully");
                    sync_from_git(scripts_dir)
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to clone scripts repository.\n\
                        Error: {}\n\
                        Repository: {}\n\
                        Target: {:?}\n\
                        Tip: Check internet connection and SCRIPTS_REPO_URL.",
                        e, remote_url, scripts_dir
                    );
                    log::error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        }
    };

    // Count after sync and append info if successful
    if let Ok(msg) = result {
        let count_after = count_scripts(scripts_dir);
        let new_count = count_after.saturating_sub(count_before);

        if new_count > 0 {
            Ok(format!("{}|new_scripts:{}", msg, new_count))
        } else {
            Ok(msg)
        }
    } else {
        result
    }
}

fn sync_from_git(repo_path: &Path) -> Result<String, String> {
    match Repository::open(repo_path) {
        Ok(repo) => {
            log::info!("Repository found, attempting sync");

            // Step 1: push any local (possibly offline-created) script changes
            // BEFORE pulling. This commits pending changes and best-effort pushes
            // them; it never hard-fails. Doing it first means scripts added while
            // offline are uploaded now that we are online, so the hard reset below
            // cannot discard them.
            if let Err(e) = script_manager::push_local_changes(repo_path) {
                log::warn!("Could not push local script changes before sync: {}", e);
            }

            let mut remote = repo
                .find_remote("origin")
                .map_err(|e| format!("Failed to find remote: {}", e))?;

            // Step 2: fetch from remote - requires internet connection.
            // NOTE: the refspec MUST include the destination
            // (`:refs/remotes/origin/main`), otherwise libgit2 only updates
            // FETCH_HEAD and leaves `refs/remotes/origin/main` pointing at the
            // commit from the very first clone. That stale ref caused two bugs:
            //   * official scripts never updated after the first download, and
            //   * freshly added user scripts (committed + pushed) were wiped by
            //     the hard reset below, because it reset back to the old commit.
            if let Err(e) = remote.fetch(&["+refs/heads/main:refs/remotes/origin/main"], None, None)
            {
                log::warn!(
                    "Failed to fetch from remote, using cached local scripts: {}",
                    e
                );
                return Ok("Using cached scripts (offline mode)".to_string());
            }

            let oid = repo
                .refname_to_id("refs/remotes/origin/main")
                .map_err(|e| format!("Failed to find remote main: {}", e))?;

            // Step 3: if the local branch still has commits that are NOT on the
            // remote (the push above failed — offline, no token, or the branches
            // diverged), do NOT hard reset: that would destroy those local-only
            // user scripts.
            if let Some(head_oid) = repo.head().ok().and_then(|head| head.target()) {
                let (ahead, behind) = repo.graph_ahead_behind(head_oid, oid).unwrap_or((0, 0));
                if ahead > 0 {
                    // If the remote also advanced (diverged history), replay the
                    // local commits on top of it. User scripts live under
                    // scripts/<namespace>/ and official scripts under official/,
                    // so they never touch the same files — the rebase is
                    // conflict-free in practice, and afterwards we push them up.
                    if behind > 0 {
                        match rebase_local_onto(&repo, oid) {
                            Ok(()) => {
                                log::info!(
                                    "Rebased {} local script commit(s) onto updated remote.",
                                    ahead
                                );
                                if let Err(e) = script_manager::push_local_changes(repo_path) {
                                    log::warn!("Rebased locally but push failed: {}", e);
                                }
                                return Ok("Scripts synced (local changes rebased)".to_string());
                            }
                            Err(e) => {
                                log::warn!(
                                    "Could not rebase local commits onto remote ({}); keeping local scripts and deferring update.",
                                    e
                                );
                                return Ok(
                                    "Local scripts preserved (pending upload); update deferred"
                                        .to_string(),
                                );
                            }
                        }
                    }

                    // Local is strictly ahead (remote did not move): nothing to
                    // pull. Keep local as-is; the push above will retry next time.
                    log::warn!(
                        "{} local script commit(s) are not yet on the remote; keeping local scripts and skipping hard reset to avoid data loss.",
                        ahead
                    );
                    return Ok(
                        "Local scripts preserved (pending upload); update deferred".to_string()
                    );
                }
            }

            let object = repo
                .find_object(oid, None)
                .map_err(|e| format!("Failed to find object: {}", e))?;

            // Safe now: local HEAD is an ancestor of (or equal to) the remote, so
            // the reset only fast-forwards in remote changes without dropping any
            // local-only commit.
            repo.reset(&object, git2::ResetType::Hard, None)
                .map_err(|e| format!("Failed to reset: {}", e))?;

            Ok("Scripts synced successfully".to_string())
        }
        Err(_) => {
            log::info!("No Git repo found at {:?}, skipping sync", repo_path);
            Err("No repository".to_string())
        }
    }
}

/// Fetches origin/main and, if the local branch is behind, rebases the
/// local-only commits on top of it. Called before publishing freshly added
/// scripts so a stale local clone does not produce a non-fast-forward
/// rejection. On conflict the rebase aborts and the working tree is left
/// untouched, so the caller can surface the failure without losing scripts.
pub(crate) fn reconcile_local_with_remote(repo_path: &Path) -> Result<(), String> {
    let repo =
        Repository::open(repo_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote: {}", e))?;

    // Same refspec as sync_from_git so refs/remotes/origin/main is actually
    // advanced (not just FETCH_HEAD) — see the note there.
    remote
        .fetch(&["+refs/heads/main:refs/remotes/origin/main"], None, None)
        .map_err(|e| format!("Failed to fetch from remote: {}", e))?;

    let oid = repo
        .refname_to_id("refs/remotes/origin/main")
        .map_err(|e| format!("Failed to resolve origin/main: {}", e))?;

    if let Some(head_oid) = repo.head().ok().and_then(|head| head.target()) {
        let (_ahead, behind) = repo.graph_ahead_behind(head_oid, oid).unwrap_or((0, 0));
        if behind == 0 {
            // Local already contains every remote commit; the push will be a
            // fast-forward. Nothing to rebase.
            return Ok(());
        }
    }

    rebase_local_onto(&repo, oid)
}

/// Replays the local-only commits on top of `onto` (the fetched origin/main),
/// using libgit2 so it also works on machines without the git CLI. Aborts and
/// returns an error on any conflict, leaving the working tree untouched, so the
/// caller can safely fall back to preserving local scripts.
pub(crate) fn rebase_local_onto(repo: &Repository, onto: git2::Oid) -> Result<(), String> {
    let signature = repo
        .signature()
        .or_else(|_| git2::Signature::now("script-runner", "script-runner@local"))
        .map_err(|e| format!("Failed to build git signature: {}", e))?;

    let head = repo
        .head()
        .map_err(|e| format!("Failed to read HEAD: {}", e))?;
    let local = repo
        .reference_to_annotated_commit(&head)
        .map_err(|e| format!("Failed to resolve HEAD commit: {}", e))?;
    let upstream = repo
        .find_annotated_commit(onto)
        .map_err(|e| format!("Failed to resolve remote commit: {}", e))?;

    let mut rebase = repo
        .rebase(Some(&local), Some(&upstream), None, None)
        .map_err(|e| format!("Failed to start rebase: {}", e))?;

    while let Some(op) = rebase.next() {
        op.map_err(|e| format!("Rebase step failed: {}", e))?;

        if repo.index().map(|idx| idx.has_conflicts()).unwrap_or(false) {
            let _ = rebase.abort();
            return Err("rebase produced conflicts".to_string());
        }

        if let Err(e) = rebase.commit(None, &signature, None) {
            // git2 returns "unmerged" as an error when a step has nothing to
            // apply; any real failure aborts to keep the tree consistent.
            let _ = rebase.abort();
            return Err(format!("Failed to commit rebased change: {}", e));
        }
    }

    rebase
        .finish(Some(&signature))
        .map_err(|e| format!("Failed to finish rebase: {}", e))?;

    Ok(())
}

#[allow(dead_code)]
pub fn list_available_scripts(scripts_dir: &PathBuf) -> Result<Vec<String>, String> {
    let mut scripts = Vec::new();

    if !scripts_dir.exists() {
        log::warn!("Scripts directory does not exist: {:?}", scripts_dir);
        return Ok(scripts);
    }

    let official_root = scripts_dir.join("official");
    let user_root = script_manager::get_user_scripts_root(scripts_dir.as_path());
    let legacy_user_root = script_manager::get_legacy_user_scripts_root(scripts_dir.as_path());

    for entry in WalkDir::new(scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let path = entry.path();
        let is_official = path.starts_with(&official_root);
        let is_current_user_script = path.starts_with(&user_root);
        let is_legacy_user_script = path.starts_with(&legacy_user_root);

        if (is_official || is_current_user_script || is_legacy_user_script)
            && (path.join("main.py").exists() || path.join("main.py.enc").exists())
        {
            if let Some(name) = entry.file_name().to_str() {
                scripts.push(name.to_string());
            }
        }
    }

    scripts.sort();
    scripts.dedup();

    log::info!("Found {} scripts", scripts.len());
    Ok(scripts)
}
