# ScriptRunner - Testing Checklist

## Overview

This document provides a comprehensive testing checklist for ScriptRunner v0.5.1. Use this to verify all features work correctly before release or after making changes.

---

## 🎨 Theme & UI Testing

### Dark/Light Mode Toggle

- [ ] **Toggle Button Visible**
  - Located in top-right corner
  - Shows sun icon in dark mode
  - Shows moon icon in light mode

- [ ] **Dark Mode**
  - Background is charcoal (#1F2937)
  - Text is white/light gray
  - Buttons have pink accent (#EC4899)
  - Cards have subtle borders
  - Glassmorphism effects visible

- [ ] **Light Mode**
  - Background is white/light gray
  - Text is dark gray
  - Buttons have pink accent (#EC4899)
  - Cards have light shadows
  - Clean, professional look

- [ ] **Transition**
  - Smooth animation (300ms)
  - No flickering
  - All components update
  - Icons transition smoothly

- [ ] **Persistence**
  - Preference saved to localStorage
  - Reloading app preserves theme
  - Works across all tabs

### Pink Theme Colors

- [ ] **Primary Pink (#EC4899)**
  - Used in buttons
  - Used in active tabs
  - Used in highlights
  - Used in progress bars

- [ ] **Gradients**
  - Hover effects on buttons
  - Card backgrounds (subtle)
  - Loading animations

- [ ] **Consistency**
  - Same pink across all components
  - No conflicting colors
  - Accessible contrast ratios (WCAG AA)

---

## 📑 Tab Navigation

### All Tabs Render Correctly

- [ ] **Dashboard (🏠)**
  - Stats cards display
  - Recent activity list
  - Quick actions visible
  - No layout issues

- [ ] **Scripts (▶️)**
  - Script list loads
  - Search bar functional
  - Filter options work
  - Script cards render

- [ ] **Execute (✨)**
  - Script selector dropdown
  - Output console visible
  - Control buttons enabled
  - Logs panel accessible

- [ ] **History (📜)**
  - Execution history loads
  - Timestamps correct
  - Filter by date works
  - Details expandable

- [ ] **Analytics (📊)**
  - Charts render without errors
  - Data loads correctly
  - Tooltips on hover
  - Responsive layout

- [ ] **Admin (🔧)**
  - Panel requires key
  - Unlock mechanism works
  - All admin sections visible
  - Proper access control

### Tab Switching

- [ ] **Smooth Transitions**
  - No lag when switching
  - Content loads immediately
  - Active tab highlighted
  - Icons update

- [ ] **State Preservation**
  - Form inputs preserved
  - Scroll position maintained
  - Filters remain active

- [ ] **Keyboard Navigation**
  - Tab key works
  - Arrow keys navigate (if applicable)
  - Enter to activate

---

## 🛡️ Admin Panel Testing

### With Admin Key

- [ ] **Key Generation**
  - "Generate Admin Key" button works
  - Key is 32 characters
  - Format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
  - Key displayed in modal
  - Copy button works

- [ ] **Unlocking Panel**
  - Paste key in input field
  - Click "Unlock Admin Panel"
  - Success notification appears
  - Panel unlocks immediately

- [ ] **Script Upload**
  - Drag & drop zone visible
  - Accepts `.py` files only
  - Validates syntax
  - Upload button functional
  - Success/error notifications

- [ ] **Admin Key Diagnostics**
  - "Show Stored Key" button works
  - Key is correct (matches generated)
  - "Copy to Clipboard" works
  - "Regenerate Admin Key" works

- [ ] **System Information**
  - App version displayed
  - Python version shown
  - Platform info correct
  - Repository URLs visible

### Without Admin Key

- [ ] **Locked State**
  - Panel shows lock icon
  - "Generate Admin Key" button visible
  - Input field for key visible
  - No sensitive info shown

- [ ] **Invalid Key Handling**
  - Error notification on wrong key
  - Panel remains locked
  - Try again option available

- [ ] **First-Time Experience**
  - Clear instructions shown
  - Generate key prompt visible
  - Help text displayed

---

## 🔔 Notification Testing

### Success Notifications

- [ ] **Script Execution Success**
  - Green notification appears
  - Success icon (✅) shown
  - Message: "Script executed successfully"
  - Auto-dismiss after 5 seconds

- [ ] **Upload Success**
  - Green notification
  - Message: "Script uploaded"
  - Copy button (if applicable)

### Error Notifications

- [ ] **Script Execution Failure**
  - Red notification appears
  - Error icon (❌) shown
  - Error message displayed
  - Manual close button

- [ ] **Network Errors**
  - Red notification
  - Descriptive error message
  - Retry action (if applicable)

### Warning Notifications

- [ ] **Missing Dependencies**
  - Yellow notification
  - Warning icon (⚠️)
  - List of missing packages
  - Install action available

- [ ] **Update Available**
  - Yellow notification
  - Update message
  - Link to download

### Info Notifications

- [ ] **General Information**
  - Blue notification
  - Info icon (ℹ️)
  - Informative message
  - Auto-dismiss

### Notification Behavior

- [ ] **Multiple Notifications**
  - Stack vertically
  - No overlapping
  - Proper spacing
  - Z-index correct

- [ ] **Position**
  - Top-right corner
  - Within viewport
  - Doesn't block UI

- [ ] **Animations**
  - Smooth slide-in
  - Fade-out on dismiss
  - Framer Motion working

- [ ] **Accessibility**
  - Screen reader compatible
  - Keyboard dismissible
  - Focus management

---

## 📊 Analytics Testing

### Chart Rendering

- [ ] **Execution Timeline (Bar Chart)**
  - Bars display correctly
  - X-axis: Dates
  - Y-axis: Execution count
  - Pink bars for success
  - Red bars for failure
  - Hover tooltips work

- [ ] **Success Rate (Pie Chart)**
  - Segments proportional
  - Green for success
  - Red for failure
  - Percentage labels
  - Center total count
  - Hover highlights

- [ ] **Top Scripts List**
  - Ranked by execution count
  - Script names correct
  - Last run timestamp
  - Click for details

### Data Accuracy

- [ ] **After Script Run**
  - Analytics update immediately
  - Execution count increments
  - Success rate recalculates
  - Charts re-render

- [ ] **Historical Data**
  - Past executions shown
  - Date range correct
  - No missing data
  - Timezone handling correct

- [ ] **Edge Cases**
  - No data: Empty state shown
  - Single data point: Chart renders
  - Large dataset: Performance OK

### Interactivity

- [ ] **Chart Tooltips**
  - Appear on hover
  - Show exact values
  - Formatted correctly

- [ ] **Click Actions**
  - Click chart → Filter/details
  - Click script → View logs

- [ ] **Responsive Design**
  - Charts resize on window resize
  - Mobile-friendly (if applicable)
  - No horizontal scroll

---

## 🐍 Script Execution Testing

### Basic Execution

- [ ] **Select Script**
  - Dropdown shows all scripts
  - Search filter works
  - Script preview available

- [ ] **Run Script**
  - "Run" button functional
  - Loading state shown
  - Output streams in real-time
  - Console updates live

- [ ] **Output Display**
  - stdout captured
  - stderr captured
  - Colors preserved (if ANSI)
  - Scrollable output

- [ ] **Completion**
  - Success message shown
  - Exit code displayed
  - Duration calculated
  - Logs saved

### Dependency Installation

- [ ] **Detect Missing Packages**
  - AST parser runs
  - Missing packages listed
  - "Install" button appears

- [ ] **Install Dependencies**
  - Pip install runs
  - Progress shown
  - Success notification
  - Script can run after

- [ ] **Already Installed**
  - No redundant installs
  - Fast execution start

### Error Handling

- [ ] **Syntax Errors**
  - Error message shown
  - Line number displayed
  - Script doesn't run

- [ ] **Runtime Errors**
  - Exception captured
  - Stack trace shown
  - Error notification

- [ ] **Timeout**
  - Long scripts can be stopped
  - Timeout configurable
  - Graceful termination

---

## ⌨️ Keyboard Shortcuts

- [ ] **Ctrl/Cmd + K**: Quick search
  - Search modal opens
  - Focus on input
  - Escape to close

- [ ] **Ctrl/Cmd + D**: Toggle dark mode
  - Theme switches
  - No delay

- [ ] **Ctrl/Cmd + R**: Refresh scripts
  - Script list reloads
  - Loading indicator shown

- [ ] **Ctrl/Cmd + ,**: Open settings
  - Settings panel opens
  - (If settings exist)

- [ ] **Esc**: Close modals
  - Any open modal closes
  - Returns focus to main UI

---

## 🔄 Auto-Update Testing

### Check for Updates

- [ ] **On Startup**
  - App checks GitHub releases
  - Compares version numbers
  - Notification if update available

- [ ] **Manual Check**
  - "Check for Updates" button
  - Loading state shown
  - Result displayed

### Download & Install

- [ ] **Download Progress**
  - Progress bar shown
  - Percentage displayed
  - Cancelable (optional)

- [ ] **Installation**
  - Prompt to restart
  - App closes and updates
  - Reopens with new version

### Version Display

- [ ] **Current Version**
  - Shown in settings/about
  - Matches package.json
  - Build date shown

---

## 🌐 Network & API Testing

### GitHub API

- [ ] **Fetch Scripts**
  - Clone/pull repository
  - Handle large repos
  - Authentication (if private)

- [ ] **Upload Scripts**
  - Push to repository
  - Commit message correct
  - Permissions verified

### Error Scenarios

- [ ] **No Internet**
  - Clear error message
  - Offline mode (if supported)
  - Retry option

- [ ] **GitHub Down**
  - Timeout gracefully
  - Fallback behavior

- [ ] **Invalid Repository**
  - Error logged
  - User-friendly message

---

## 💾 Data Persistence

### Local Storage

- [ ] **Theme Preference**
  - Saved on toggle
  - Loaded on startup
  - Survives app restart

- [ ] **Admin Key**
  - Saved securely
  - Loaded on unlock attempt
  - Persists across sessions

### SQLite Database (Analytics)

- [ ] **Database Creation**
  - Created on first run
  - Schema correct
  - Located in app data folder

- [ ] **Data Insertion**
  - Executions recorded
  - Timestamps accurate
  - Foreign keys enforced

- [ ] **Data Retrieval**
  - Queries performant
  - No duplicate data
  - Sorted correctly

### File System

- [ ] **Script Cache**
  - Scripts downloaded to local folder
  - Updates on sync
  - Old scripts removed

- [ ] **Logs**
  - Execution logs saved
  - One file per execution
  - Timestamped filenames
  - Rotation (if applicable)

---

## 📱 Responsive Design

### Window Resizing

- [ ] **Minimum Width**
  - UI doesn't break at 800px
  - Scrollbars appear if needed

- [ ] **Maximum Width**
  - Content centered (if designed)
  - No overflow

- [ ] **Height Changes**
  - Vertical scrolling works
  - Footer/header fixed (if designed)

### Component Layout

- [ ] **Cards**
  - Stack on narrow screens
  - Side-by-side on wide screens

- [ ] **Modals**
  - Centered on all screen sizes
  - Scrollable if content overflows

---

## 🐛 Error & Edge Cases

### Empty States

- [ ] **No Scripts**
  - Empty state message
  - "Sync Scripts" button
  - Helpful instructions

- [ ] **No Analytics Data**
  - Empty chart placeholders
  - "Run a script to see analytics"

- [ ] **No History**
  - "No executions yet" message

### Invalid Input

- [ ] **Invalid Admin Key**
  - Error notification
  - Clear error message
  - Try again option

- [ ] **Invalid Script File**
  - Upload validation fails
  - Error message explains why

### Performance

- [ ] **Large Script Output**
  - Doesn't freeze UI
  - Virtualized scrolling (if implemented)
  - Truncate if > 10MB

- [ ] **Many Scripts (100+)**
  - List renders quickly
  - Search is fast
  - Pagination (if implemented)

---

## 🔧 Build & Deployment

### Development Build

- [ ] **`npm run tauri dev`**
  - Starts without errors
  - Hot reload works
  - Console shows no errors

### Production Build

- [ ] **`npm run tauri build`**
  - Build completes successfully
  - No TypeScript errors
  - No Rust warnings
  - Binary created in `target/release`

### Installer

- [ ] **Windows (.exe)**
  - Installer runs
  - App installs to Program Files
  - Desktop shortcut created
  - Uninstaller works

- [ ] **macOS (.dmg)**
  - DMG opens
  - Drag to Applications works
  - App launches from Applications
  - No security warnings (if signed)

---

## ✅ Final Checklist

Before Release:

- [ ] All tests above passed
- [ ] No console errors
- [ ] No console warnings (critical ones)
- [ ] Documentation updated
- [ ] Changelog created
- [ ] Version number incremented
- [ ] Git tag created
- [ ] GitHub release published
- [ ] Installer tested on clean machine
- [ ] Code reviewed
- [ ] Security audit (if applicable)
- [ ] Performance profiling done
- [ ] User acceptance testing
- [ ] Backup created

---

## 📝 Test Results Template

**Date**: _______  
**Tester**: _______  
**Version**: 0.5.1  
**Platform**: Windows / macOS  

**Results**:
- ✅ Passed: ___ / ___
- ❌ Failed: ___ / ___
- ⚠️ Warnings: ___

**Issues Found**:
1. 
2. 
3. 

**Notes**:


**Sign-off**: _______

---

## 🔄 Automated Testing (Future)

**Coming Soon:**
- [ ] Unit tests (Vitest)
- [ ] Integration tests (Playwright)
- [ ] E2E tests (Tauri test suite)
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Code coverage reports
- [ ] Performance benchmarks

---

**Last Updated**: February 2026  
**Version**: 0.5.1  
