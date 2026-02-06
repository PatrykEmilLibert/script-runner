# GitHub Admin Authentication Guide

## Overview

ScriptRunner now supports **GitHub-based admin authentication**. This is **completely optional** - the app works perfectly fine without logging in. However, if you want access to the Admin Panel, you need to log in with GitHub and be on the approved admin list.

## How It Works

1. **Login with GitHub Personal Access Token (PAT)**
   - You provide a GitHub token with access to the private `script-runner-config` repository
   - ScriptRunner verifies your identity with GitHub
   - It checks if your GitHub username is in the `admins.json` file

2. **Admin List Management**
   - Admins are managed in a private GitHub repository: `script-runner-config`
   - The file `admins.json` contains the list of approved admin usernames
   - Only users in this list can access the Admin Panel

3. **No Login Required for Normal Use**
   - You can run scripts, view history, and use all basic features without logging in
   - Admin Panel is only for advanced management (official scripts, diagnostics, etc.)

---

## Step-by-Step: Generate GitHub PAT

### 1. Go to GitHub Settings

- Open GitHub: https://github.com
- Click your profile picture (top-right) → **Settings**
- Scroll down in the left sidebar → **Developer settings**

### 2. Create Personal Access Token

- Click **Personal access tokens** → **Tokens (classic)**
- Click **Generate new token (classic)**

### 3. Configure Token

Fill in the form:

- **Note**: `ScriptRunner Admin` (or any name you prefer)
- **Expiration**: Choose expiration (30 days, 90 days, or no expiration)
- **Select scopes**: Check **`repo`** (Full control of private repositories)
  - This is required to access the private `script-runner-config` repo

### 4. Generate and Copy

- Click **Generate token** at the bottom
- **⚠️ IMPORTANT**: Copy the token immediately! You won't see it again.
- The token looks like: `ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`

### 5. Use in ScriptRunner

- Open ScriptRunner
- Go to **Dashboard** tab
- Find the **GitHub Login** card
- Click **Login with GitHub Personal Access Token**
- Paste your token
- Click **Login**

If your GitHub username is in `admins.json`, you'll see:
- ✅ Green badge: **Admin**
- Access to **Admin Panel** tab

---

## Managing Admins

### Add a New Admin

1. Edit `admins.json` in the `script-runner-config` repository
2. Add a new entry:

```json
{
  "version": 1,
  "admins": [
    {
      "github_username": "PatrykEmilLibert",
      "role": "owner",
      "added_at": "2026-02-06T00:00:00Z",
      "notes": "Project creator"
    },
    {
      "github_username": "new-admin-username",
      "role": "admin",
      "added_at": "2026-02-06T12:00:00Z",
      "notes": "New team member"
    }
  ],
  "updated_at": "2026-02-06T12:00:00Z"
}
```

3. Commit and push the changes
4. The new admin can now log in with their GitHub token

### Remove an Admin

1. Edit `admins.json`
2. Remove the user's entry
3. Commit and push
4. The user will lose admin access immediately

---

## Security Best Practices

### ✅ DO

- **Keep your token private** - never share it publicly
- Use **short expiration** (30-90 days) and regenerate regularly
- Store the token in a **password manager**
- Use **minimal scopes** (only `repo` is needed)
- Review who has admin access regularly

### ❌ DON'T

- **Never commit** the token to Git
- Don't share your token with anyone
- Don't publish it in screenshots or docs
- Don't use the same token for multiple apps

---

## Troubleshooting

### "Login failed: Invalid token"
- Token may be expired or invalid
- Regenerate a new token on GitHub

### "Login successful, but you are not an admin"
- Your GitHub username is not in `admins.json`
- Contact the repository owner to add you

### "Failed to fetch admin config"
- The token doesn't have `repo` scope
- The `script-runner-config` repository is missing
- Check your internet connection

---

## Legacy Admin Key

The old `sr-admin.key` system is still supported as a **fallback**. If you have an admin key file on your Desktop, it will work even without GitHub login.

However, we recommend **switching to GitHub authentication** for:
- ✅ Easier admin management
- ✅ No manual key distribution
- ✅ Centralized access control
- ✅ Instant revocation (just remove from `admins.json`)

---

## Quick Links

- [Generate GitHub Token](https://github.com/settings/tokens/new)
- [GitHub Token Documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
- [Script Runner Config Repo](https://github.com/PatrykEmilLibert/script-runner-config)

---

**Questions?** Open an issue on the ScriptRunner repository.
