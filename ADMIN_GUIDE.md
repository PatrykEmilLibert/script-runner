# Admin Panel - Complete Guide

## Overview

The Admin Panel provides advanced management features for ScriptRunner administrators. It requires authentication via an admin key and offers control over script management and system diagnostics.

---

## 🔑 Getting Started

### Generating Your First Admin Key

1. Launch ScriptRunner
2. Navigate to the **Admin** tab (🔧 icon)
3. Click **"Generate Admin Key"** button
4. **Important**: Copy and save the key immediately!
   - The key is 32 characters long
   - Format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
   - Store in password manager or secure location

5. Paste the key in the input field
6. Click **"Unlock Admin Panel"**

### Security Best Practices

- ✅ **Never share** your admin key
- ✅ **Store securely** in password manager
- ✅ **Regenerate** if compromised
- ✅ **Use different keys** for different admins
- ❌ **Don't hardcode** in scripts
- ❌ **Don't commit** to version control
- ❌ **Don't send** via email/chat

---

## 📤 Script Management

### Uploading Scripts

**Method 1: Drag & Drop** (Recommended)

1. Unlock Admin Panel
2. Scroll to **"Upload Scripts"** section
3. Drag a `.py` file to the dropzone
4. File is validated automatically
5. Click **"Upload to Repository"**
6. Success notification confirms upload

**Method 2: Click to Browse**

1. Click on the dropzone
2. Select `.py` file from file picker
3. Click **"Upload to Repository"**

**Validation Rules**:
- ✅ Must be `.py` file
- ✅ File size < 10MB
- ✅ Valid Python syntax (checked via AST)
- ✅ No malicious imports (configurable blocklist)
- ❌ Rejects non-Python files
- ❌ Rejects files with syntax errors

**Where Files Go**:
- Uploaded to `SCRIPTS_REPO_URL/scripts/` directory
- Automatically committed with message: `"Admin upload: [filename]"`
- Pushed to GitHub immediately
- Available to all users after next sync

### Removing Scripts

**Manual Method** (Current):
1. Go to your scripts repository on GitHub
2. Navigate to `scripts/` folder
3. Delete unwanted script
4. Commit changes
5. Users' instances will remove on next sync

**Via Admin Panel** (Coming in v0.6.0):
- One-click script removal
- Confirmation dialog
- Audit log entry

### Updating Scripts

**Method 1: Via Admin Panel**
1. Upload new version with same filename
2. Overwrites existing script
3. Users get update on next sync

**Method 2: Via GitHub**
1. Edit script directly on GitHub
2. Commit changes
3. Users pull changes on sync

---

## 🔍 Admin Key Diagnostics

### Viewing Stored Key

1. Unlock Admin Panel
2. Scroll to **"Admin Key Diagnostics"**
3. Click **"Show Stored Key"**
4. Key is displayed with copy button

### Regenerating Key

**When to regenerate:**
- Lost or forgotten key
- Key compromised
- Security audit requirement
- Periodic rotation policy

**Steps:**
1. Go to Admin Panel (even if locked)
2. Click **"Regenerate Admin Key"**
3. **Warning**: Old key becomes invalid immediately
4. New key is generated and displayed
5. Copy and store securely
6. All other admins need new key

### Copying Key

1. Click **"Copy to Clipboard"** button
2. Paste in password manager
3. Share securely with authorized admins

---

## 📊 System Information

The Admin Panel displays:

### Application Info
- **Version**: Current app version (e.g., `0.5.1`)
- **Build Date**: When binary was compiled
- **Environment**: Development / Production

### Python Environment
- **Python Version**: Embedded Python version (e.g., `3.12.9`)
- **Pip Version**: Package manager version
- **Installed Packages**: Count of installed packages
- **Virtual Environment**: Active venv path

### Platform Details
- **OS**: Windows / macOS / Linux
- **Architecture**: x64 / ARM64
- **User**: Current username
- **Hostname**: Machine name

### Configuration
- **Scripts Repository**: GitHub URL for scripts
- **Scripts Directory**: Local path to cached scripts
- **Logs Directory**: Local path to execution logs
- **Analytics DB**: Path to SQLite database

---

## 📈 Analytics Interpretation

### Execution Statistics

**Metrics Available:**
1. **Total Executions**: Lifetime script runs
2. **Success Rate**: `(successful / total) * 100`
3. **Average Duration**: Mean execution time
4. **Scripts per Day**: Rolling 7-day average
5. **Most Used Script**: Highest execution count

**Reading Charts:**

1. **Execution Timeline (Bar Chart)**
   - X-axis: Date
   - Y-axis: Number of executions
   - Pink bars: Successful runs
   - Red bars: Failed runs
   - Hover for exact count

2. **Success Rate (Pie Chart)**
   - Green segment: Successful
   - Red segment: Failed
   - Percentage labels
   - Total count in center

3. **Top Scripts (List)**
   - Ranked by execution count
   - Shows last run timestamp
   - Click for detailed logs

### Performance Analysis

**Identifying Issues:**
- ⚠️ **High Failure Rate** (>10%): Check script errors in logs
- ⚠️ **Long Execution Times**: Optimize slow scripts
- ⚠️ **Unusual Spikes**: Investigate automated runs
- ⚠️ **Zero Executions**: Scripts may be unused (consider removal)

**Actionable Insights:**
1. **Most Failed Script**: Prioritize debugging
2. **Slowest Script**: Candidate for optimization
3. **Most Used Script**: Ensure high reliability
4. **Unused Scripts**: Archive or document

### Exporting Data

**Current Method** (manual):
1. Locate `analytics.db` in app data folder
2. Use SQLite browser to query
3. Export to CSV

**Coming Soon** (v0.6.0):
- One-click CSV export
- JSON export
- Email reports
- Scheduled summaries

---

## 🛠️ Troubleshooting

### Admin Panel Won't Unlock

**Symptom**: "Invalid admin key" error

**Solutions**:
1. Verify key is exactly 32 characters
2. Check for leading/trailing spaces
3. Regenerate key and try again
4. Check browser console for errors
5. Restart app and retry

### Script Upload Fails

**Symptom**: Upload button shows error

**Solutions**:
1. Verify file is valid Python (`.py` extension)
2. Check file size < 10MB
3. Ensure no syntax errors (run locally first)
4. Verify repository write permissions
5. Check network connectivity

### Analytics Not Recording

**Symptom**: Charts show no data

**Solutions**:
1. Run a script and check if data appears
2. Verify `analytics.db` exists in app data folder
3. Check file permissions (read/write)
4. Restart app to reinitialize DB
5. Check console for SQLite errors

### Notifications Not Appearing

**Symptom**: No success/error messages

**Solutions**:
1. Check if Mantine provider is configured
2. Look for console errors
3. Verify notification position (may be off-screen)
4. Disable browser notification blockers
5. Restart app

---

## 🔐 Security Considerations

### Admin Key Storage

**Where it's stored:**
- **Windows**: `%APPDATA%\ScriptRunner\config.json`
- **macOS**: `~/Library/Application Support/ScriptRunner/config.json`
- **Linux**: `~/.config/ScriptRunner/config.json`

**Encryption:**
- Key is hashed with SHA-256
- Compared with user input hash
- Never transmitted over network
- Not logged or tracked

### Script Upload Security

**Risks:**
- Malicious scripts can be uploaded
- Scripts run with user's permissions
- Can access local filesystem

**Mitigation:**
1. Review all uploaded scripts manually
2. Use automated scanning (coming soon)
3. Implement approval workflow
4. Blocklist dangerous imports (`os.system`, `subprocess`, etc.)
5. Sandbox execution environment (roadmap)

---

## 📋 Admin Checklist

### Daily Tasks
- [ ] Review analytics for anomalies
- [ ] Monitor notification logs
- [ ] Verify backup scripts sync

### Weekly Tasks
- [ ] Review top failing scripts
- [ ] Analyze performance trends
- [ ] Update documentation
- [ ] Test admin panel access

### Monthly Tasks
- [ ] Rotate admin keys
- [ ] Audit script repository
- [ ] Clean up unused scripts
- [ ] Export analytics reports
- [ ] Review security logs

### Quarterly Tasks
- [ ] Security audit
- [ ] Update dependencies
- [ ] Performance optimization review
- [ ] User feedback collection

---

## 🆘 Emergency Procedures

### Revoking Compromised Admin Key

**Scenario**: Admin key leaked

**Actions** (10-minute response):
1. Generate new admin key immediately
2. Distribute to authorized admins only
3. Notify team of key rotation
4. Audit recent admin actions
5. Review logs for unauthorized access

### Script Removal (Urgent)

**Scenario**: Malicious script detected

**Actions** (15-minute response):
1. Remove script from GitHub repository
2. Force sync on all client machines
3. Scan for damage/data exfiltration
4. Notify affected users

---

## 📚 Additional Resources

- **Main Docs**: [README.md](README.md)
- **Upgrade Guide**: [UPGRADE_GUIDE.md](UPGRADE_GUIDE.md)
- **Testing**: [TESTING.md](TESTING.md)
- **API Reference**: Coming soon
- **Video Tutorials**: Coming soon

---

**Last Updated**: February 2026  
**Version**: 0.5.1  
**Maintainer**: Admin Team
