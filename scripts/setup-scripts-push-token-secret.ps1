param(
    [Parameter(Mandatory = $true)]
    [string]$Token,
    [string]$Repository = "PatrykEmilLibert/script-runner"
)

$ErrorActionPreference = "Stop"

$gh = "C:\Program Files\GitHub CLI\gh.exe"
if (-not (Test-Path $gh)) {
    $gh = "gh"
}

$tokenBytes = [System.Text.Encoding]::UTF8.GetBytes($Token)
$tokenB64 = [System.Convert]::ToBase64String($tokenBytes)

Write-Host "Setting GitHub Actions secret SR_SCRIPTS_PUSH_TOKEN_B64 for $Repository..."
$tokenB64 | & $gh secret set SR_SCRIPTS_PUSH_TOKEN_B64 --repo $Repository --body -

Write-Host "Done. Secret SR_SCRIPTS_PUSH_TOKEN_B64 has been updated."