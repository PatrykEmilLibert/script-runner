# 🚀 GitHub Setup & Deployment Guide

## Krok 1: Inicjalizacja Git Repository

```bash
cd P:\python_runner_github\script-runner

# Inicjalizuj repo
git init

# Dodaj wszystkie pliki
git add .

# Pierwszy commit
git commit -m "Initial commit: ScriptRunner - Python Script Executor with Tauri + React"
```

## Krok 2: Stwórz GitHub Repository

### A. Przez przeglądarkę (GitHub.com):

1. Zaloguj się do GitHub
2. Kliknij **"+"** → **"New repository"**
3. Nazwa: `script-runner` lub `python-script-executor`
4. Opis: `Cross-platform Python script executor with auto-updates, kill switch, and dependency management`
5. **⚠️ NIE zaznaczaj:** "Initialize with README" (masz już pliki)
6. Wybierz **Private** lub **Public** (zalecam Private dla enterprise)
7. Kliknij **"Create repository"**

### B. Połącz lokalne repo z GitHub:

```bash
# Zastąp YOUR_USERNAME swoją nazwą użytkownika GitHub
git remote add origin https://github.com/YOUR_USERNAME/script-runner.git

# Rename branch to main (jeśli trzeba)
git branch -M main

# Push pierwszy raz
git push -u origin main
```

## Krok 3: Stwórz Kill Switch Repository

Kill switch wymaga osobnego PRYWATNEGO repo dla bezpieczeństwa:

### A. Stwórz nowe repo:

1. GitHub → **"New repository"**
2. Nazwa: `script-runner-config` (MUSI być private!)
3. **Private** ✅ (WAŻNE!)
4. Initialize with README ✅
5. Create

### B. Dodaj kill_switch.json:

```bash
# Sklonuj repo config
cd P:\python_runner_github
git clone https://github.com/YOUR_USERNAME/script-runner-config.git
cd script-runner-config

# Stwórz kill switch file
echo '{"blocked": false}' > kill_switch.json

# Commit i push
git add kill_switch.json
git commit -m "Add kill switch configuration"
git push
```

## Krok 4: GitHub Personal Access Token (PAT)

Potrzebny do:
- Kill switch sprawdzania (publiczny dostęp)
- GitHub Actions (auto-build)
- Auto-updates

### Stwórz token:

1. GitHub → **Settings** → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
2. **"Generate new token (classic)"**
3. Nazwa: `ScriptRunner Kill Switch`
4. Expiration: **No expiration** (lub 1 rok)
5. Scopes:
   - ✅ `repo` (pełny dostęp do prywatnych repo)
   - ✅ `read:org` (jeśli używasz organization)
6. **Generate token**
7. **⚠️ SKOPIUJ TOKEN TERAZ** - nie zobaczysz go ponownie!

## Krok 5: Zaktualizuj Kod z Twoimi Danymi

### A. .env file (lokalne testy):

```bash
cd P:\python_runner_github\script-runner

# Stwórz .env
@"
SCRIPTS_REPO_URL=https://github.com/YOUR_USERNAME/python-scripts
KILL_SWITCH_REPO=YOUR_USERNAME/script-runner-config
GITHUB_TOKEN=ghp_YourPersonalAccessTokenHere
"@ | Out-File -FilePath .env -Encoding UTF8
```

### B. Zaktualizuj kill_switch.rs:

Otwórz `src-tauri/src/kill_switch.rs` i zmień URL:

```rust
// Linia ~10
.get("https://api.github.com/repos/YOUR_USERNAME/script-runner-config/contents/kill_switch.json")
```

Zamień `YOUR_USERNAME` na swoją nazwę użytkownika GitHub.

## Krok 6: GitHub Actions Secrets

Dla automatycznego budowania i releasów:

1. GitHub → Twoje repo `script-runner` → **Settings** → **Secrets and variables** → **Actions**
2. **New repository secret** dla każdego:

```
GITHUB_TOKEN (automatyczny - już istnieje)
```

Dla macOS codesigning (opcjonalne):
```
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
APPLE_SIGNING_IDENTITY
APPLE_ID
APPLE_PASSWORD
```

## Krok 7: Stwórz Scripts Repository

Jeśli chcesz synchronizować skrypty z GitHub:

```bash
cd P:\python_runner_github
mkdir python-scripts
cd python-scripts

git init
echo "# Python Scripts Repository" > README.md

# Dodaj example skrypty
cp ../script-runner/examples/*.py .

git add .
git commit -m "Initial scripts"

# Stwórz repo na GitHub i push
git remote add origin https://github.com/YOUR_USERNAME/python-scripts.git
git push -u origin main
```

## Krok 8: Testuj Kill Switch

```bash
# 1. Sprawdź czy aplikacja działa
npm run tauri dev

# 2. Zmień kill_switch.json w repo config
cd P:\python_runner_github\script-runner-config
echo '{"blocked": true}' > kill_switch.json
git add kill_switch.json
git commit -m "Test: Block application"
git push

# 3. Uruchom aplikację ponownie - powinna być zablokowana!

# 4. Odblokuj
echo '{"blocked": false}' > kill_switch.json
git add kill_switch.json
git commit -m "Unblock application"
git push
```

## Krok 9: GitHub Releases dla Auto-Update

### A. Tag pierwszego release:

```bash
cd P:\python_runner_github\script-runner

git tag -a v0.1.0 -m "First release - ScriptRunner v0.1.0"
git push origin v0.1.0
```

### B. GitHub Actions automatycznie:

- Zbuduje Windows .exe
- Zbuduje macOS .dmg
- Stworzy GitHub Release z instalatorami

## Krok 10: Zabezpieczenia

### ⚠️ NIGDY NIE COMMITUJ:

```
❌ Personal Access Tokens
❌ API Keys
❌ .env files (już w .gitignore)
❌ Prywatnych danych użytkowników
❌ Haseł
```

### ✅ ZAWSZE:

```
✅ Używaj .env dla secrets
✅ GitHub Secrets dla CI/CD
✅ Private repo dla kill switch
✅ Review code przed commit
✅ Używaj signed commits (opcjonalne)
```

### Sprawdź .gitignore:

```bash
# Sprawdź co zostanie zacommitowane
git status

# Zobacz co jest ignorowane
git check-ignore -v node_modules/
```

## Krok 11: Signed Commits (Opcjonalne - Bezpieczeństwo++)

```bash
# Generate GPG key
gpg --full-generate-key

# List keys
gpg --list-secret-keys --keyid-format LONG

# Export public key
gpg --armor --export YOUR_KEY_ID

# Dodaj do GitHub: Settings → SSH and GPG keys → New GPG key

# Configure git
git config --global user.signingkey YOUR_KEY_ID
git config --global commit.gpgsign true
```

## Krok 12: Workflow dla Zespołu

### Dla developerów:

```bash
# Clone repo
git clone https://github.com/YOUR_USERNAME/script-runner.git
cd script-runner

# Setup
npm install
npm run tauri dev

# Praca nad feature
git checkout -b feature/new-feature
# ... edycja ...
git add .
git commit -m "feat: Add new feature"
git push origin feature/new-feature

# Stwórz Pull Request na GitHub
```

### Dla użytkowników końcowych:

```
1. Idź do: https://github.com/YOUR_USERNAME/script-runner/releases
2. Pobierz najnowszy .exe (Windows) lub .dmg (Mac)
3. Zainstaluj
4. Aplikacja automatycznie sprawdzi kill switch i sync scripts
```

## Podsumowanie - Quick Start

```powershell
# 1. Git init i push
cd P:\python_runner_github\script-runner
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/YOUR_USERNAME/script-runner.git
git push -u origin main

# 2. Stwórz kill switch repo (przez GitHub.com)
# 3. Clone i dodaj kill_switch.json
cd ..
git clone https://github.com/YOUR_USERNAME/script-runner-config.git
cd script-runner-config
echo '{"blocked": false}' > kill_switch.json
git add . && git commit -m "Add kill switch" && git push

# 4. Zaktualizuj kill_switch.rs z Twoim username
# 5. Tag release
cd ../script-runner
git tag v0.1.0
git push origin v0.1.0

# 6. GitHub Actions zbuduje .exe automatycznie!
```

---

## 🆘 Troubleshooting

### "Push rejected - authentication failed"

```bash
# Użyj Personal Access Token jako hasła
# Username: YOUR_GITHUB_USERNAME
# Password: ghp_YourTokenHere

# Lub skonfiguruj credential manager
git config --global credential.helper manager-core
```

### "Kill switch nie działa"

1. Sprawdź czy repo `script-runner-config` jest **private**
2. Sprawdź URL w `kill_switch.rs`
3. Sprawdź format JSON: `{"blocked": false}` (bez spacji)
4. Test: `curl https://api.github.com/repos/YOUR_USERNAME/script-runner-config/contents/kill_switch.json`

### "GitHub Actions build fails"

1. Sprawdź czy tag jest pushed: `git push origin v0.1.0`
2. Sprawdź Actions tab na GitHub
3. Zobacz logi dla szczegółów błędu

---

**Gotowe! Masz teraz profesjonalny workflow z GitHub** 🎉
