# Trigger build-release workflow (requires gh CLI)
# Run: gh auth login (if not authenticated)

$ErrorActionPreference = "Stop"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host "Install GitHub CLI: https://cli.github.com/" -ForegroundColor Red
    Write-Host "  winget install GitHub.cli" -ForegroundColor Yellow
    exit 1
}

$repo = gh repo view --json nameWithOwner -q .nameWithOwner 2>$null
if (-not $repo) {
    Write-Host "Repo not found. Run from project root with git init." -ForegroundColor Red
    Write-Host "  gh repo create --source=. --push" -ForegroundColor Yellow
    exit 1
}

Write-Host "Running build-release for $repo..." -ForegroundColor Cyan
gh workflow run "build-release.yml"

if ($LASTEXITCODE -eq 0) {
    Write-Host "Workflow started. Artifacts at:" -ForegroundColor Green
    Write-Host "  https://github.com/$repo/actions" -ForegroundColor Cyan
    Write-Host "Download AppImage: Actions -> latest run -> Artifacts -> KengaIDE-Linux-AppImage" -ForegroundColor Yellow
} else {
    exit 1
}
