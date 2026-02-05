# Kill Switch - Rozbudowa Backend (Rust)

## 📋 Podsumowanie zmian

Rozbudowano system Kill Switch o zaawansowane funkcje zarządzania, whitelistę, blokowanie czasowe, cache offline i pełne API administracyjne.

---

## 🆕 Nowe struktury danych

### `KillSwitchConfig`
```rust
pub struct KillSwitchConfig {
    pub blocked: bool,                  // Czy aplikacja jest zablokowana
    pub reason: String,                 // Powód blokady
    pub blocked_until: Option<String>,  // Timestamp ISO 8601 końca blokady
    pub whitelist: Vec<String>,         // Lista ID maszyn z dostępem
    pub message: String,                // Wiadomość dla użytkownika
    pub redirect_url: Option<String>,   // URL przekierowania
    pub cached_at: Option<String>,      // Timestamp cache (wewnętrzne)
}
```

**Przykład JSON:**
```json
{
  "blocked": true,
  "reason": "Maintenance in progress",
  "blocked_until": "2026-02-10T18:00:00Z",
  "whitelist": [
    "DESKTOP-ABC123-john-a1b2c3d4",
    "MacBook-XYZ-admin-e5f6g7h8"
  ],
  "message": "Scheduled maintenance. Back at 6 PM UTC.",
  "redirect_url": "https://scriptrunner.example.com/status"
}
```

---

## 🔧 Nowe funkcje Rust

### **kill_switch.rs**

#### `check_remote_status_advanced() -> Result<KillSwitchConfig>`
- Pobiera pełną konfigurację Kill Switch z GitHub
- Automatycznie cachuje odpowiedź lokalnie
- Graceful degradation do cache gdy GitHub niedostępny
- Timeout: 5 sekund

#### `is_machine_whitelisted(config: &KillSwitchConfig) -> bool`
- Sprawdza czy obecna maszyna jest na whiteliście
- Porównuje z wynikiem `get_machine_id()`
- Whitelist bypass wszystkie blokady

#### `should_block(config: &KillSwitchConfig) -> bool`
- Główna logika decyzji o blokowaniu
- Sprawdza whitelist → czas blokady → status blocked
- Zwraca `true` jeśli aplikacja powinna być zablokowana

#### `get_machine_id() -> String`
- Generuje/pobiera unikalny ID maszyny
- Format: `{hostname}-{username}-{uuid}`
- Zapisywany w `AppData/ScriptRunner/cache/machine_id.txt`
- Stały dla danej maszyny i użytkownika

#### `cache_kill_switch_status(config: &KillSwitchConfig) -> Result<()>`
- Zapisuje konfigurację lokalnie
- Ścieżka: `AppData/ScriptRunner/cache/kill_switch_cache.json`
- Dodaje timestamp `cached_at`

#### `get_cached_status() -> Option<KillSwitchConfig>`
- Odczytuje cache lokalny
- Waliduje wiek cache (24h dla fresh, ale zwraca zawsze w offline)
- Zwraca `None` jeśli brak cache lub błąd parsowania

---

### **kill_switch_manager.rs** (NOWY PLIK)

#### `toggle_kill_switch(blocked: bool, reason: String) -> Result<String>`
- Włącza/wyłącza Kill Switch globalnie
- Wymaga `GITHUB_TOKEN` w środowisku
- Pushuje zmiany przez GitHub API
- **Admin only**

#### `schedule_block(until: String, reason: String) -> Result<String>`
- Scheduluje czasową blokadę
- `until`: timestamp ISO 8601 (np. `"2026-02-10T12:00:00Z"`)
- Automatycznie włącza `blocked = true`
- **Admin only**

#### `add_to_whitelist(machine_id: String) -> Result<String>`
- Dodaje machine ID do whitelisty
- Sprawdza duplikaty
- Pushuje przez GitHub API
- **Admin only**

#### `remove_from_whitelist(machine_id: String) -> Result<String>`
- Usuwa machine ID z whitelisty
- Bezpieczne jeśli ID nie istnieje
- **Admin only**

#### `set_custom_message(message: String, redirect_url: Option<String>) -> Result<String>`
- Ustawia własną wiadomość dla zablokowanych użytkowników
- Opcjonalny URL przekierowania
- **Admin only**

#### `push_config_to_github(config: KillSwitchConfig) -> Result<()>`
- Wysyła zaktualizowaną konfigurację na GitHub
- Automatycznie pobiera SHA poprzedniej wersji
- Koduje JSON do base64 (wymagane przez GitHub API)
- Loguje wszystkie operacje

#### `create_local_override(allow: bool) -> Result<String>`
- **EMERGENCY**: Lokalne nadpisanie Kill Switch
- Ważne przez 1 godzinę
- Zapisywane w cache, sprawdzane jako pierwsze
- **Admin only**

#### `check_local_override() -> Option<bool>`
- Sprawdza czy istnieje aktywny lokalny override
- Usuwa expired overrides automatycznie
- Zwraca `Some(true)` jeśli allow, `Some(false)` jeśli block

---

## 🎯 Nowe Tauri Commands

### Dla użytkowników:

#### `check_kill_switch() -> Result<bool, String>`
- **Backward compatible** - zwraca tylko bool
- Sprawdza local override → remote config
- `true` = zablokowane, `false` = dozwolone

#### `check_kill_switch_status() -> Result<KillSwitchConfig, String>`
- Zwraca pełną konfigurację
- Zawiera message, redirect_url, whitelist
- Frontend może wyświetlić szczegóły blokady

#### `get_kill_switch_status() -> Result<KillSwitchConfig, String>`
- Alias `check_kill_switch_status` (kompatybilność wsteczna)

#### `get_current_machine_id() -> Result<String, String>`
- Zwraca ID obecnej maszyny
- Użytkownik może przekazać adminowi do whitelisty
- Nie wymaga admin key

---

### Dla adminów (wymagają `admin_key`):

#### `toggle_kill_switch_cmd(blocked: bool, reason: String, admin_key: String) -> Result<String>`
- Włącz/wyłącz Kill Switch globalnie
- Wymaga `GITHUB_TOKEN` w environment

#### `schedule_kill_switch(until: String, reason: String, admin_key: String) -> Result<String>`
- Zaplanuj czasową blokadę
- Format `until`: `"2026-02-10T18:00:00Z"`

#### `add_machine_to_whitelist(machine_id: String, admin_key: String) -> Result<String>`
- Dodaj maszynę do whitelisty

#### `remove_machine_from_whitelist(machine_id: String, admin_key: String) -> Result<String>`
- Usuń maszynę z whitelisty

#### `set_kill_switch_message(message: String, redirect_url: Option<String>, admin_key: String) -> Result<String>`
- Ustaw własną wiadomość dla użytkowników

#### `create_kill_switch_override(allow: bool, admin_key: String) -> Result<String>`
- **EMERGENCY**: Lokalny override (1h)
- Bypass zdalnej konfiguracji

---

## 🛡️ Obsługa błędów i bezpieczeństwo

### Graceful Degradation
```
GitHub dostępny → Pobierz + cache
GitHub niedostępny → Użyj cache (jeśli <24h)
Brak cache → Default (allow app to run)
```

### Cache offline
- **Ważność**: 24 godziny dla "fresh" statusu
- **Fallback**: Nawet expired cache używany gdy offline
- **Lokalizacja**: `AppData/ScriptRunner/cache/`

### Logi
- Wszystkie operacje Kill Switch logowane
- Poziomy: `info`, `warn`, `error`
- Widoczne w konsoli dev i production logs

### Admin override
- Tylko z valid admin key
- Czas życia: 1 godzina
- Automatyczne usuwanie po wygaśnięciu
- Logowane jako `WARN`

---

## 📦 Zależności dodane do Cargo.toml

```toml
base64 = "0.21"      # Kodowanie dla GitHub API
whoami = "1.4"       # Hostname/username dla machine ID
```

---

## 🚀 Przykłady użycia

### Frontend - Sprawdzenie statusu
```typescript
const status = await invoke<KillSwitchConfig>('check_kill_switch_status');

if (status.blocked && !status.whitelist.includes(myMachineId)) {
  if (status.blocked_until) {
    showMessage(`Blocked until ${status.blocked_until}: ${status.message}`);
  } else {
    showMessage(status.message);
    if (status.redirect_url) {
      window.location.href = status.redirect_url;
    }
  }
}
```

### Frontend - Pobranie Machine ID
```typescript
const machineId = await invoke<string>('get_current_machine_id');
console.log('This machine ID:', machineId);
// Użytkownik może wysłać adminowi do whitelisty
```

### Admin Panel - Toggle Kill Switch
```typescript
await invoke('toggle_kill_switch_cmd', {
  blocked: true,
  reason: 'Critical security update required',
  adminKey: userAdminKey
});
```

### Admin Panel - Zaplanuj blokadę
```typescript
const until = new Date('2026-02-10T18:00:00Z').toISOString();
await invoke('schedule_kill_switch', {
  until,
  reason: 'Scheduled maintenance',
  adminKey: userAdminKey
});
```

### Admin Panel - Whitelist maszyny
```typescript
await invoke('add_machine_to_whitelist', {
  machineId: 'DESKTOP-ABC123-john-a1b2c3d4',
  adminKey: userAdminKey
});
```

### Emergency - Local Override
```typescript
// Pozwól uruchomić aplikację lokalnie przez 1h (bypass GitHub)
await invoke('create_kill_switch_override', {
  allow: true,
  adminKey: userAdminKey
});
```

---

## 🔑 Setup dla admina

### 1. Wygeneruj GitHub Personal Access Token
```
GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
Scopes: repo (full control)
```

### 2. Ustaw environment variable
**Windows:**
```powershell
$env:GITHUB_TOKEN = "ghp_your_token_here"
```

**Linux/Mac:**
```bash
export GITHUB_TOKEN="ghp_your_token_here"
```

### 3. Struktura repo GitHub
```
script-runner-config/
├── kill_switch.json    ← Edytowany przez API
└── README.md
```

---

## ✅ Testy

### Test 1: Normalna sytuacja
- GitHub dostępny, `blocked: false` → App działa

### Test 2: Blokada globalna
- `blocked: true` → Wszyscy zablokowani

### Test 3: Whitelist
- `blocked: true`, machine w `whitelist` → App działa

### Test 4: Blokada czasowa
- `blocked_until` w przyszłości → Zablokowane
- `blocked_until` przeszłe → App działa

### Test 5: Offline mode
- GitHub niedostępny, cache <24h → Używa cache
- Brak cache → Default (allow)

### Test 6: Local override
- Override aktywny → Ignoruje GitHub
- Override expired → Normalny flow

---

## 📚 Pełna lista funkcji

### kill_switch.rs (9 funkcji)
1. `check_remote_status()` - legacy bool check
2. `check_remote_status_advanced()` - pełny config
3. `is_machine_whitelisted()` - sprawdza whitelist
4. `should_block()` - logika blokowania
5. `get_machine_id()` - ID maszyny
6. `cache_kill_switch_status()` - zapis cache
7. `get_cached_status()` - odczyt cache
8. `get_cache_dir()` - helper path
9. `KillSwitchConfig` struct + Default impl

### kill_switch_manager.rs (10 funkcji)
1. `toggle_kill_switch()` - włącz/wyłącz
2. `schedule_block()` - zaplanuj blokadę
3. `add_to_whitelist()` - dodaj do whitelisty
4. `remove_from_whitelist()` - usuń z whitelisty
5. `set_custom_message()` - ustaw wiadomość
6. `fetch_current_config()` - pobierz z GitHub
7. `push_config_to_github()` - wyślij na GitHub
8. `create_local_override()` - emergency override
9. `check_local_override()` - sprawdź override
10. Stałe: `GITHUB_API_BASE`, `CACHE_DURATION_HOURS`

### main.rs (10 nowych commands)
1. `check_kill_switch()` - bool check
2. `check_kill_switch_status()` - full status
3. `get_kill_switch_status()` - alias
4. `toggle_kill_switch_cmd()` - admin toggle
5. `schedule_kill_switch()` - admin schedule
6. `add_machine_to_whitelist()` - admin whitelist add
7. `remove_machine_from_whitelist()` - admin whitelist remove
8. `get_current_machine_id()` - user machine ID
9. `set_kill_switch_message()` - admin message
10. `create_kill_switch_override()` - admin emergency

---

**ŁĄCZNIE: 29 nowych funkcji + 1 nowa struktura + 10 Tauri commands**
