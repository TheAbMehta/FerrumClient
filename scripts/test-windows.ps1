#!/usr/bin/env pwsh

Write-Host "=== Running tests on Windows (DX12) ===" -ForegroundColor Cyan

$ErrorActionPreference = "Stop"

Write-Host "Running cargo test..." -ForegroundColor Yellow
cargo test --workspace --verbose

if ($LASTEXITCODE -ne 0) {
    Write-Host "Tests failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "=== All tests passed on Windows ===" -ForegroundColor Green
