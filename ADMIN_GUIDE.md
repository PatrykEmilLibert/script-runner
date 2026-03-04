# Admin Panel - Complete Guide

## Overview

The Admin Panel provides advanced management features for ScriptRunner administrators.
Access is controlled only by **GitHub admin authentication** (PAT + `admins.json` in `script-runner-config`).

---

## 🔑 Getting Started

### GitHub Admin Login

1. Launch ScriptRunner
2. Use **GitHub Login** (drawer/button)
3. Paste a GitHub Personal Access Token (PAT)
4. Ensure your GitHub username exists in `admins.json`
5. Open the **Admin** tab (🔧)

If your account is not in `admins.json`, admin features remain locked.

### Required PAT Access

- Token must be valid (not expired/revoked)
- Token must allow reading private `script-runner-config` repository
- For admin mutations (kill switch/config), token must have write access

---

## 🔐 Security Best Practices

- ✅ Keep PAT in a password manager
- ✅ Use short token expiration and rotate periodically
- ✅ Remove unused admins from `admins.json`
- ✅ Audit admin actions regularly
- ❌ Never share PAT over chat/email
- ❌ Never commit PAT to repository
- ❌ Never expose PAT in screenshots

---

## 📤 Script Management

### Uploading Scripts

**Method 1: Drag & Drop** (Recommended)

1. Open **Admin** tab
2. Scroll to **Upload Scripts** section
3. Drag `.py` file to dropzone
4. Confirm upload
5. Script is pushed to repository

**Validation Rules**:
- ✅ Must be `.py` file
- ✅ File size < 10MB
- ✅ Valid Python syntax
- ❌ Rejects non-Python files

### Bulk Management

In **Official Scripts** section you can:
- Select many scripts with checkboxes
- `Encrypt Selected`
- `Edit Metadata` (author/version/description prefix)
- `Delete Selected` with confirmation

### Updating Scripts

- Replace code for a single script
- Full edit of metadata + content
- Bulk metadata update for multiple scripts

---

## ⚙️ Kill Switch Administration

Available for logged-in GitHub admins:

- Toggle kill switch on/off
- Schedule block window
- Manage whitelist entries
- Set custom blocked message
- Create local override (admin-only emergency action)

All operations are validated against current GitHub admin status.

---

## 📊 System Information

The Admin Panel displays:

### Application Info
- Version
- Environment

### Python Environment
- Python version
- Installed dependencies status

### Platform Details
- OS
- Architecture

### Configuration
- Scripts repository URL
- Local scripts directory
- Logs path

---

## 🛠️ Troubleshooting

### Admin Panel Locked

**Symptom**: Admin tab actions unavailable

**Check:**
1. Confirm PAT is valid and not expired
2. Confirm PAT can access `script-runner-config`
3. Confirm your username is listed in `admins.json`
4. Re-login and refresh admin status

### Script Upload Fails

1. Verify `.py` extension
2. Check repository write permissions
3. Verify network/GitHub availability
4. Review app logs

### Kill Switch Action Fails

1. Verify you are still logged in as GitHub admin
2. Ensure token has required repo permissions
3. Check connectivity to config repository

---

## 📋 Admin Checklist

### Daily
- [ ] Monitor failed script runs
- [ ] Review recent admin actions

### Weekly
- [ ] Review top failing scripts
- [ ] Validate kill switch configuration
- [ ] Verify admin list in `admins.json`

### Monthly
- [ ] Rotate PATs (recommended)
- [ ] Audit repository permissions
- [ ] Review inactive admins

---

## 🆘 Emergency Procedures

### Revoke Admin Access

**Scenario**: Compromised admin account/token

1. Remove user from `admins.json`
2. Commit and push config changes
3. Invalidate compromised PAT in GitHub settings
4. Verify user can no longer perform admin actions

### Urgent Script Removal

1. Delete script from official repository
2. Trigger sync in clients
3. Confirm script is no longer runnable

---

## 📚 Additional Resources

- [README.md](README.md)
- [GITHUB_AUTH_GUIDE.md](GITHUB_AUTH_GUIDE.md)
- [UPGRADE_GUIDE.md](UPGRADE_GUIDE.md)
- [TESTING.md](TESTING.md)

---

**Last Updated**: March 2026  
**Auth Model**: GitHub-only admin access
