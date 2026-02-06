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
4. Opis: `Cross-platform Python script executor with auto-updates and dependency management`
5. **⚠️ NIE zaznaczaj:** "Initialize with README" (masz już pliki)
6. Wybierz **Private** lub **Public** (zalecam Private dla enterprise)
7. Kliknij **"Create repository"**

### B. Połącz lokalne repo z GitHub:

```bash
# URL Twojego repo
git remote add origin https://github.com/PatrykEmilLibert/script-runner.git

# Rename branch to main (jeśli trzeba)
git branch -M main

# Push pierwszy raz
git push -u origin main
```

## Krok 3: GitHub Personal Access Token (PAT)

Potrzebny do:
- GitHub Actions (auto-build)
- Auto-updates

### Stwórz token:

1. GitHub → **Settings** → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
2. **"Generate new token (classic)"**
3. Nazwa: `ScriptRunner GitHub Access`
4. Expiration: **No expiration** (lub 1 rok)
5. Scopes:
   - ✅ `repo` (pełny dostęp do prywatnych repo)
   - ✅ `read:org` (jeśli używasz organization)
6. **Generate token**
7. **⚠️ SKOPIUJ TOKEN TERAZ** - nie zobaczysz go ponownie!

## Krok 4: Zaktualizuj Kod z Twoimi Danymi

### A. .env file (lokalne testy):

```bash
cd P:\python_runner_github\script-runner

# Stwórz .env
@"
SCRIPTS_REPO_URL=https://github.com/PatrykEmilLibert/python-scripts
GITHUB_TOKEN=ghp_YourPersonalAccessTokenHere
"@ | Out-File -FilePath .env -Encoding UTF8
```

## Krok 5: GitHub Actions Secrets

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

## Krok 6: Stwórz Scripts Repository

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
git remote add origin https://github.com/PatrykEmilLibert/python-scripts.git
git push -u origin main
```

## Krok 7: GitHub Releases dla Auto-Update

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

## Krok 8: Zabezpieczenia

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

## Krok 9: Signed Commits (Opcjonalne - Bezpieczeństwo++)

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

## Krok 10: Workflow dla Zespołu

### Dla developerów:

```bash
# Clone repo
git clone https://github.com/PatrykEmilLibert/script-runner.git
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
1. Idź do: https://github.com/PatrykEmilLibert/script-runner/releases
2. Pobierz najnowszy .exe (Windows) lub .dmg (Mac)
3. Zainstaluj
4. Aplikacja automatycznie zsynchronizuje skrypty
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

# 2. Stwórz scripts repo (przez GitHub.com)
# 3. Clone i dodaj scripts
cd ..
git clone https://github.com/PatrykEmilLibert/python-scripts.git
cd python-scripts
# dodaj swoje skrypty
git add . && git commit -m "Add scripts" && git push

# 4. Tag release
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

### "GitHub Actions build fails"

1. Sprawdź czy tag jest pushed: `git push origin v0.1.0`
2. Sprawdź Actions tab na GitHub
3. Zobacz logi dla szczegółów błędu

---

**Gotowe! Masz teraz profesjonalny workflow z GitHub** 🎉
