# ScriptRunner - Deployment Checklist

Use this checklist when deploying ScriptRunner to your team.

## Pre-Release ✅

- [ ] All tests passing
- [ ] No console errors in dev
- [ ] README updated
- [ ] CHANGELOG updated
- [ ] Version bumped in `package.json`
- [ ] Version bumped in `src-tauri/Cargo.toml`
- [ ] `.env.example` is current
- [ ] No hardcoded credentials in code

## Build Process ✅

- [ ] Run `npm run tauri build` on Windows
- [ ] Run `npm run tauri build` on Mac
- [ ] Verify `.exe` file size reasonable
- [ ] Verify `.dmg` file size reasonable
- [ ] Test Windows installer
- [ ] Test Mac installer

## Release ✅

- [ ] Create GitHub release with tag `v1.0.0`
- [ ] Add release notes (copy from CHANGELOG)
- [ ] Upload `.exe` to release
- [ ] Upload `.dmg` to release
- [ ] Test download links work
- [ ] Enable auto-updates in app

## Documentation ✅

- [ ] README has correct repository URLs
- [ ] Setup guide is accurate
- [ ] Quick start guide is clear
- [ ] Examples are runnable
- [ ] Links in docs are correct

## Infrastructure ✅

- [ ] Scripts repository created
- [ ] Kill switch repository created
- [ ] `kill_switch.json` initialized
- [ ] GitHub Actions workflows set up
- [ ] All environment variables configured
- [ ] Logging directory exists

## Team Distribution ✅

- [ ] Create team wiki/docs
- [ ] Share download links
- [ ] Explain how to use
- [ ] Share GitHub access (if private repos)
- [ ] Set up support channel
- [ ] Train team members

## Monitoring ✅

- [ ] Check logs for errors
- [ ] Monitor usage patterns
- [ ] Gather feedback
- [ ] Document common issues
- [ ] Plan improvements

## Maintenance ✅

- [ ] Schedule regular updates
- [ ] Review security patches
- [ ] Update dependencies
- [ ] Archive old logs
- [ ] Backup configurations

## Troubleshooting

### If build fails:
- Check Rust version: `rustc --version`
- Check Node version: `node --version`
- Clear cache: `cargo clean`
- Try on clean machine

### If deployment stalls:
- Check internet connection
- Verify all URLs are accessible
- Test GitHub API access
- Check firewall rules

### If app doesn't run:
- Check Windows/Mac version compatibility
- Run in admin mode (Windows)
- Check permissions
- Review application logs

## Rollback Plan

If issues in production:

1. **For app update:**
   - Create `kill_switch.json` with `blocked: true`
   - Push to GitHub immediately
   - All apps will stop within seconds

2. **For scripts:**
   - Revert bad script in repo
   - Push fix
   - Apps will auto-sync

3. **For full rollback:**
   - Release previous version
   - Update `SCRIPTS_REPO_URL` to previous commit
   - Notify users

## Success Criteria

✅ **Deployment successful when:**
- Users can download app
- App installs without errors
- Scripts run without manual intervention
- Dependencies install automatically
- Team gives positive feedback
- No critical bugs reported

---

**Questions?** Check [SETUP.md](SETUP.md) or open an issue!
