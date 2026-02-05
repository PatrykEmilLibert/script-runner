# ScriptRunner Upgrade Guide

## Version 0.5.1 - Pink Theme & Analytics Edition

Welcome to the biggest update of ScriptRunner! This guide will help you migrate from previous versions and understand all the new features.

---

## 🎉 What's New

### 🌸 1. Pink Theme System
**The entire UI has been redesigned with a beautiful pink color palette!**

- **Primary Color**: `#EC4899` (pink-500)
- **Dark Mode**: Charcoal backgrounds with pink accents
- **Light Mode**: Clean white backgrounds with pink highlights
- **Smooth Transitions**: All theme changes are animated
- **Toggle**: Click the sun/moon icon in the top-right corner

**Benefits:**
- Modern, professional look
- Better contrast and readability
- Consistent branding across all components
- Eye-friendly color schemes for both modes

### 📊 2. Analytics Dashboard
**Track your script usage with beautiful charts!**

New tab: **Analytics** (📊 icon)

Features:
- **Execution Timeline**: Bar chart showing daily script runs
- **Success Rate**: Pie chart with success/failure ratio
- **Top Scripts**: List of most frequently used scripts
- **Performance Metrics**: Average execution time
- **Real-time Updates**: Stats update after each script run

**Data Tracked:**
- Script name
- Execution timestamp
- Success/failure status
- Execution duration
- User information

**Charts powered by Recharts** - interactive, responsive, and beautiful!

### 🛡️ 3. Admin Panel
**Advanced management features for administrators!**

New tab: **Admin** (🔧 icon)

**Access Control:**
- Requires admin key authentication
- Key generated via "Generate Admin Key" button
- 32-character secure random key
- Stored in system config

**Features:**
1. **Kill Switch Management**
   - View current status (Blocked/Active)
   - Last check timestamp
   - Manual refresh button
   - Visual status indicators

2. **Script Upload (Drag & Drop)**
   - Drag `.py` files directly to the panel
   - Automatic validation
   - Upload to configured repository
   - Success/error notifications

3. **Admin Key Diagnostics**
   - View stored admin key
   - Regenerate if lost
   - Copy to clipboard
   - Security warnings

4. **System Information**
   - App version
   - Python version
   - Platform info
   - Script repository URL

### 🔔 4. Smart Notification System
**Beautiful, contextual notifications for all actions!**

**Types:**
- ✅ **Success** (green) - Script completed, upload successful
- ❌ **Error** (red) - Execution failed, network errors
- ⚠️ **Warning** (yellow) - Missing dependencies, updates available
- ℹ️ **Info** (blue) - General information, status updates

**Features:**
- Auto-dismiss after 5 seconds
- Manual close button
- Positioned at top-right
- Smooth animations (Framer Motion)
- Icon per notification type
- Stack multiple notifications

**Powered by Mantine Notifications** - accessible and customizable!

### 🎨 5. Enhanced UI Components
**Every component has been polished!**

- **Tabs**: New icons from `lucide-react` (Sparkles, Play, History, etc.)
- **Buttons**: Pink gradient on hover, smooth transitions
- **Cards**: Glassmorphism effects with backdrop blur
- **Inputs**: Pink focus states, better validation
- **Loading States**: Pink spinners and progress bars
- **Modals**: Centered overlays with blur backgrounds

### ⌨️ 6. Keyboard Shortcuts
**Power user features!**

- `Ctrl/Cmd + K` - Quick script search
- `Ctrl/Cmd + D` - Toggle dark mode
- `Ctrl/Cmd + R` - Refresh scripts
- `Ctrl/Cmd + ,` - Open settings
- `Esc` - Close modals

---

## 🔄 Breaking Changes

### ⚠️ **None!**
This update is **100% backward compatible**. Your existing:
- Scripts continue to work
- Logs are preserved
- Configuration stays the same
- Kill switch configuration unchanged

### ⚡ What Changed Internally:
1. **Theme System**: New Zustand store for theme state
2. **Analytics**: SQLite database for tracking (auto-created)
3. **Notifications**: Replaced custom alerts with Mantine
4. **Icons**: Migrated to `lucide-react` from `@tabler/icons-react`

---

## 📦 Migration Steps

### Step 1: Update Dependencies
```bash
cd script-runner
npm install
```

**New packages added:**
- `recharts@^2.10.0` - Charts library
- `framer-motion@^10.16.0` - Animation library
- `@mantine/core@^7.4.0` - UI components
- `@mantine/hooks@^7.4.0` - React hooks
- `lucide-react@^0.300.0` - Icon library
- `date-fns@^3.0.0` - Date utilities

### Step 2: Update Rust Dependencies
```bash
cd src-tauri
cargo update
```

**New crates:**
- `rusqlite@0.30` - SQLite database
- `machine-uid@0.3` - Hardware ID generation

### Step 3: Rebuild Application
```bash
npm run tauri build
```

### Step 4: First Launch
On first launch:
1. **Analytics DB** will be auto-created in app data folder
2. **Theme** defaults to dark mode (your preference is saved)
3. **Admin Key** - Generate if you want admin access

---

## 📸 Screenshots

### Pink Theme (Dark Mode)
```
[Placeholder: Screenshot showing dark mode with pink accents]
- Charcoal background (#1F2937)
- Pink buttons and highlights (#EC4899)
- Smooth gradients
```

### Analytics Dashboard
```
[Placeholder: Analytics tab with charts]
- Bar chart of executions over time
- Pie chart of success rate
- Top scripts list
```

### Admin Panel
```
[Placeholder: Admin panel with kill switch]
- Admin key input
- Kill switch status card
- Drag & drop upload zone
```

### Notifications
```
[Placeholder: Notification stack]
- Success notification (green)
- Error notification (red)
- Warning notification (yellow)
```

---

## 🆕 New Features Guide

### Using Analytics
1. Go to **Analytics** tab (📊)
2. View charts and metrics
3. Click on chart elements for details
4. Data updates automatically after each run
5. Export analytics (coming soon)

### Accessing Admin Panel
1. Go to **Admin** tab (🔧)
2. Click "Generate Admin Key" (first time only)
3. **Save the key securely!**
4. Enter key to unlock panel
5. Manage kill switch, upload scripts, view diagnostics

### Customizing Theme
1. Click sun/moon icon (top-right)
2. Theme toggles between dark/light
3. Preference is saved locally
4. Smooth transition animation

### Managing Notifications
- Notifications appear automatically on actions
- Click `X` to dismiss manually
- Auto-dismiss after 5 seconds
- Multiple notifications stack vertically

---

## ❓ FAQ

### Q: Will my old scripts work?
**A:** Yes! 100% backward compatible. All scripts work exactly as before.

### Q: How do I get the admin key?
**A:** Go to Admin tab → "Generate Admin Key" → Copy and save it securely.

### Q: Where is analytics data stored?
**A:** In SQLite database at:
- **Windows**: `%APPDATA%\ScriptRunner\analytics.db`
- **Mac**: `~/Library/Application Support/ScriptRunner/analytics.db`

### Q: Can I export analytics?
**A:** Coming in v0.6.0! For now, data is stored in SQLite (can be queried manually).

### Q: How do I change theme color from pink?
**A:** Edit `src/theme.ts` and rebuild. Custom themes coming in future updates.

### Q: What if I lose the admin key?
**A:** Go to Admin tab → "Admin Key Diagnostics" → "Regenerate Admin Key"

### Q: Do notifications slow down the app?
**A:** No! Notifications are lightweight and use hardware acceleration.

### Q: Can I disable analytics?
**A:** Yes, toggle in Settings (coming soon). For now, data is only stored locally.

### Q: Is my data sent to servers?
**A:** **No!** All analytics are stored locally on your machine. Zero telemetry.

### Q: How do I upload scripts via Admin Panel?
**A:**
1. Unlock Admin Panel with key
2. Scroll to "Upload Scripts" section
3. Drag `.py` file to the dropzone
4. Click "Upload to Repository"

### Q: What icons are used?
**A:** `lucide-react` - modern, consistent, open-source icons.

---

## 🔧 Troubleshooting

### Charts not displaying?
1. Check if `recharts` is installed: `npm list recharts`
2. Rebuild: `npm run tauri build`
3. Clear browser cache (if dev mode)

### Admin Panel won't unlock?
1. Check if admin key is correct (32 characters)
2. Regenerate key if lost
3. Check browser console for errors

### Theme toggle not working?
1. Check if `zustand` is installed
2. Clear local storage and reload
3. Check `theme.ts` for errors

### Notifications not showing?
1. Verify `@mantine/notifications` is installed
2. Check if `MantineProvider` wraps app
3. Look for console errors

---

## 🎯 Next Steps

After upgrading:
1. ✅ Explore Analytics dashboard
2. ✅ Generate admin key
3. ✅ Try the new pink theme
4. ✅ Test drag & drop upload
5. ✅ Review notification system
6. ✅ Check keyboard shortcuts

**Enjoy the new ScriptRunner!** 🌸✨

For issues or questions:
- GitHub Issues: [your-repo/issues]
- Documentation: `README.md`, `ADMIN_GUIDE.md`
- Testing: `TESTING.md`

---

**Version:** 0.5.1  
**Release Date:** February 2026  
**Codename:** Pink Analytics  
