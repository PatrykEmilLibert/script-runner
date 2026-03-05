param(
    [Parameter(Mandatory = $true)]
    [string]$PfxPath,

    [Parameter(Mandatory = $true)]
    [string]$PfxPassword,

    [Parameter(Mandatory = $false)]
    [string]$Repo = "PatrykEmilLibert/script-runner"
)

if (-not (Test-Path $PfxPath)) {
    throw "PFX file not found: $PfxPath"
}

$gh = Get-Command gh -ErrorAction SilentlyContinue
if (-not $gh) {
    throw "GitHub CLI (gh) is required. Install gh first."
}

if (-not $env:GH_TOKEN) {
    $credIn = "protocol=https`nhost=github.com`n`n"
    $credOut = $credIn | git credential fill
    $tokenLine = ($credOut | Select-String '^password=').Line
    if ($tokenLine) {
        $env:GH_TOKEN = ($tokenLine -replace '^password=', '')
    }
}

if (-not $env:GH_TOKEN) {
    throw "No GH_TOKEN found. Run 'gh auth login' or set GH_TOKEN."
}

$pfxBase64 = [Convert]::ToBase64String([System.IO.File]::ReadAllBytes((Resolve-Path $PfxPath)))

$pfxBase64 | gh secret set WINDOWS_CERTIFICATE --repo $Repo
$PfxPassword | gh secret set WINDOWS_CERTIFICATE_PASSWORD --repo $Repo

gh api "repos/$Repo/actions/secrets/WINDOWS_CERTIFICATE" --jq ".name"
gh api "repos/$Repo/actions/secrets/WINDOWS_CERTIFICATE_PASSWORD" --jq ".name"

Write-Host "Windows code-signing secrets configured for $Repo"