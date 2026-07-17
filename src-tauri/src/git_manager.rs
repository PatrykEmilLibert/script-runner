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

            // Step 0: discard the working-tree side effects of local official-
            // script encryption BEFORE anything else, so they can never enter a
            // sync commit and permanently diverge this clone from the remote.
            restore_official_encryption_artifacts(&repo, repo_path);

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
                                // The rebase could not be applied cleanly. This
                                // is the freeze an old client hits when a local
                                // official-script encryption commit (from before
                                // Step 0 existed) collides with any upstream edit
                                // to the same script — a modify/delete conflict
                                // that recurs on every sync, so official scripts
                                // never update again. Recover by resetting to the
                                // remote while preserving genuine user scripts.
                                log::warn!(
                                    "Could not rebase local commits onto remote ({}); attempting recovery reset.",
                                    e
                                );
                                match recover_diverged_branch(&repo, repo_path, oid) {
                                    Ok(()) => {
                                        return Ok(
                                            "Scripts synced (recovered from diverged history)"
                                                .to_string(),
                                        );
                                    }
                                    Err(re) => {
                                        log::warn!(
                                            "Recovery reset failed ({}); keeping local scripts and deferring update.",
                                            re
                                        );
                                        return Ok(
                                            "Local scripts preserved (pending upload); update deferred"
                                                .to_string(),
                                        );
                                    }
                                }
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

/// Discards the working-tree side effects of local official-script encryption
/// so they never enter a sync commit.
///
/// After every sync the app encrypts official scripts in place
/// (`official/<name>/main.py` -> `main.py.enc`, deleting the plaintext — see
/// `script_encryption::encrypt_official_scripts`). Those are modifications to
/// git-tracked files. If the pre-sync `push_local_changes` commits them, this
/// clone's branch permanently diverges from the remote, and every later upstream
/// edit to an existing official script becomes a modify/delete conflict that
/// freezes the whole library. Restoring the official tree to HEAD before syncing
/// keeps the branch clean while leaving the on-disk `.enc` files (re-created
/// right after each sync) as the local, never-committed encryption layer.
fn restore_official_encryption_artifacts(repo: &Repository, repo_path: &Path) {
    // 1. Bring tracked files back to HEAD (restores any deleted main.py).
    let mut checkout = git2::build::CheckoutBuilder::new();
    checkout.force();
    if let Err(e) = repo.checkout_head(Some(&mut checkout)) {
        log::warn!("Could not restore tracked files to HEAD before sync: {}", e);
        return;
    }

    // 2. Remove the untracked *.enc artifacts. A locally-encrypted script now
    //    has BOTH main.py (just restored) and main.py.enc; a script encrypted on
    //    the remote has only main.py.enc (tracked) and no main.py — so this
    //    discriminator only ever deletes local artifacts, never remote content.
    let official_root = repo_path.join("official");
    if !official_root.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(&official_root) {
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let main_py = dir.join("main.py");
            let main_enc = dir.join("main.py.enc");
            if main_py.exists() && main_enc.exists() {
                if let Err(e) = std::fs::remove_file(&main_enc) {
                    log::warn!(
                        "Could not remove local encryption artifact {:?}: {}",
                        main_enc,
                        e
                    );
                }
            }
        }
    }
}

/// Last-resort recovery when the local branch has diverged from origin/main in a
/// way we cannot cleanly rebase (an old client that committed local official-
/// script encryption before Step 0 existed). Preserves genuine user scripts,
/// hard-resets to origin/main so official scripts match the remote again, then
/// restores any user scripts that only existed locally so they are never lost.
fn recover_diverged_branch(
    repo: &Repository,
    repo_path: &Path,
    onto: git2::Oid,
) -> Result<(), String> {
    log::warn!("Recovering diverged scripts repo: reset to origin/main, preserving user scripts.");

    let scripts_root = repo_path.join("scripts");
    let backup_root = std::env::temp_dir().join("script_runner_user_scripts_backup");

    // 1. Back up the whole user scripts tree (all namespaces + legacy layout).
    let _ = std::fs::remove_dir_all(&backup_root);
    if scripts_root.exists() {
        copy_tree(&scripts_root, &backup_root, true)
            .map_err(|e| format!("Failed to back up user scripts: {}", e))?;
    }

    // 2. Hard reset to origin/main so official scripts match the remote again.
    let object = repo
        .find_object(onto, None)
        .map_err(|e| format!("Failed to find remote object: {}", e))?;
    repo.reset(&object, git2::ResetType::Hard, None)
        .map_err(|e| format!("Failed to reset to remote: {}", e))?;

    // 3. Restore only the user script files the reset dropped (local-only work);
    //    never overwrite the versions the remote already has.
    let mut restored = 0usize;
    if backup_root.exists() {
        restored = copy_tree(&backup_root, &scripts_root, false)
            .map_err(|e| format!("Failed to restore local user scripts: {}", e))?;
    }
    let _ = std::fs::remove_dir_all(&backup_root);

    // 4. Re-commit the restored user scripts so they get published (best-effort).
    if restored > 0 {
        log::info!(
            "Restored {} local user script file(s) after recovery reset.",
            restored
        );
        if let Err(e) = script_manager::push_local_changes(repo_path) {
            log::warn!(
                "Restored {} user script file(s) but could not publish them yet: {}",
                restored,
                e
            );
        }
    }

    Ok(())
}

/// Recursively copies `src` into `dest`. When `overwrite` is false, existing
/// destination files are left untouched (used to restore only the local-only
/// user scripts a recovery reset would otherwise drop). Returns the number of
/// files actually written.
fn copy_tree(src: &Path, dest: &Path, overwrite: bool) -> std::io::Result<usize> {
    let mut copied = 0usize;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = match entry.path().strip_prefix(src) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        let target = dest.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if !overwrite && target.exists() {
                continue;
            }
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
            copied += 1;
        }
    }

    Ok(copied)
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
