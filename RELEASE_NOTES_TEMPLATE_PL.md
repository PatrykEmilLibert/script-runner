# Szablon Release Notes (PL)

## ScriptRunner {{WERSJA}}
Data wydania: {{DATA}}

### Co nowego
- {{PUNKT_1}}
- {{PUNKT_2}}
- {{PUNKT_3}}

### Pobieranie
- Windows MSI: `ScriptRunner_*.msi`
- Windows NSIS: `ScriptRunner_*_x64-setup.exe`
- Windows Portable: `ScriptRunner-Portable.exe`
- macOS: `ScriptRunner_*.dmg`
- macOS pomocniczy starter: `START_MAC.command`

### Szybka instrukcja macOS (unsigned build)
1. Pobierz `ScriptRunner_*.dmg` oraz `START_MAC.command`.
2. Otwórz DMG i przeciągnij `ScriptRunner.app` do `Applications`.
3. Uruchom `START_MAC.command` (jeśli dwuklik nie działa: `bash START_MAC.command`).

### Znane ograniczenia
- Wersja macOS bez Apple Developer signing/notarization może wymagać uruchomienia `START_MAC.command`.

### Aktualizacja
- W aplikacji: uruchom synchronizację skryptów.
- Skrypty pobierają się z repozytorium zdalnego (nie są bundlowane z aplikacją).

### Zgłaszanie problemów
- GitHub Issues: {{LINK_DO_ISSUES}}
