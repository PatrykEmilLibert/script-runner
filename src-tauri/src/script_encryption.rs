use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use std::fs;
use std::path::Path;

const ENCRYPTION_KEY: &[u8; 32] = b"ScriptRunnerSecureKey2024!@#$%^&"; // In production: use per-install unique key
const NONCE_SIZE: usize = 12;

/// Encrypt a Python script file
pub fn encrypt_script(script_path: &Path) -> Result<(), String> {
    // Read original content
    let content = fs::read_to_string(script_path)
        .map_err(|e| format!("Failed to read script: {}", e))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let cipher = Aes256Gcm::new_from_slice(ENCRYPTION_KEY)
        .map_err(|e| format!("Cipher error: {}", e))?;
    
    let encrypted = cipher
        .encrypt(nonce, content.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Combine nonce + encrypted data
    let mut output = nonce_bytes.to_vec();
    output.extend_from_slice(&encrypted);

    // Write encrypted file with .enc extension
    let enc_path = script_path.with_extension("py.enc");
    fs::write(&enc_path, output)
        .map_err(|e| format!("Failed to write encrypted file: {}", e))?;

    // Delete original
    fs::remove_file(script_path)
        .map_err(|e| format!("Failed to remove original: {}", e))?;

    Ok(())
}

/// Decrypt and return script content (in-memory only, never writes to disk)
pub fn decrypt_script(enc_path: &Path) -> Result<String, String> {
    // Read encrypted file
    let data = fs::read(enc_path)
        .map_err(|e| format!("Failed to read encrypted script: {}", e))?;

    if data.len() < NONCE_SIZE {
        return Err("Invalid encrypted file (too short)".to_string());
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let cipher = Aes256Gcm::new_from_slice(ENCRYPTION_KEY)
        .map_err(|e| format!("Cipher error: {}", e))?;
    
    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed (file corrupted or tampered): {}", e))?;

    String::from_utf8(decrypted)
        .map_err(|e| format!("Invalid UTF-8 in decrypted content: {}", e))
}

/// Check if a script is encrypted
#[allow(dead_code)]
pub fn is_encrypted(script_dir: &Path) -> bool {
    script_dir.join("main.py.enc").exists()
}

/// Get the appropriate script path (encrypted or plain)
#[allow(dead_code)]
pub fn get_script_path(script_dir: &Path) -> Option<std::path::PathBuf> {
    let enc_path = script_dir.join("main.py.enc");
    let plain_path = script_dir.join("main.py");
    
    if enc_path.exists() {
        Some(enc_path)
    } else if plain_path.exists() {
        Some(plain_path)
    } else {
        None
    }
}
