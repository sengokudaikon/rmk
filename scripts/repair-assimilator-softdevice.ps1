param(
    [string]$BackupDir = ".\keeb-info\softdevice-repair",
    [switch]$Apply
)

$ErrorActionPreference = "Stop"

function Get-Uf2DriveInfo {
    Get-PSDrive -PSProvider FileSystem | ForEach-Object {
        $root = $_.Root
        $infoPath = Join-Path $root "INFO_UF2.TXT"
        $currentPath = Join-Path $root "CURRENT.UF2"
        if (-not (Test-Path -LiteralPath $infoPath)) {
            return
        }

        $info = Get-Content -LiteralPath $infoPath -Raw
        if ($info -notmatch "Board-ID:\s+nRF52840-assimilator-ble") {
            return
        }

        [PSCustomObject]@{
            Drive       = $_.Name
            Root        = $root
            InfoPath    = $infoPath
            CurrentPath = $currentPath
            Info        = $info
            HasCurrent  = Test-Path -LiteralPath $currentPath
            HasS140     = $info -match "SoftDevice:\s+S140\s+6\.1\.1"
            MissingSd   = $info -match "SoftDevice:\s+not found"
        }
    }
}

$root = Resolve-Path -LiteralPath $PSScriptRoot\..
$backupPath = Join-Path $root $BackupDir
New-Item -ItemType Directory -Force -Path $backupPath | Out-Null

$drives = @(Get-Uf2DriveInfo)

if ($drives.Count -eq 0) {
    Write-Error "No nRF52840-assimilator-ble UF2 bootloader drives found. Put both halves in bootloader mode and run again."
}

$drives | Select-Object Drive,Root,HasCurrent,HasS140,MissingSd | Format-Table -AutoSize

$donor = $drives | Where-Object { $_.HasS140 -and $_.HasCurrent } | Select-Object -First 1
$target = $drives | Where-Object { $_.MissingSd } | Select-Object -First 1

if (-not $donor) {
    Write-Error "Could not find a donor half with SoftDevice S140 6.1.1 and CURRENT.UF2."
}

if (-not $target) {
    Write-Error "Could not find a target half reporting 'SoftDevice: not found'."
}

$donorBackup = Join-Path $backupPath "donor-$($donor.Drive)-CURRENT.UF2"
$targetInfoBackup = Join-Path $backupPath "target-$($target.Drive)-INFO_UF2.TXT"
Copy-Item -LiteralPath $donor.CurrentPath -Destination $donorBackup -Force
Copy-Item -LiteralPath $target.InfoPath -Destination $targetInfoBackup -Force

Write-Host ""
Write-Host "Donor:  $($donor.Root) has S140 6.1.1"
Write-Host "Target: $($target.Root) reports SoftDevice not found"
Write-Host "Backed up donor CURRENT.UF2 to: $donorBackup"
Write-Host "Backed up target INFO_UF2.TXT to: $targetInfoBackup"

if (-not $Apply) {
    Write-Host ""
    Write-Host "Dry run only. To clone the donor CURRENT.UF2 onto the missing-SoftDevice half, run:"
    Write-Host "  pwsh -ExecutionPolicy Bypass -File .\scripts\repair-assimilator-softdevice.ps1 -Apply"
    exit 0
}

Write-Host ""
Write-Host "Writing donor CURRENT.UF2 to target $($target.Root). The target should reboot/eject when done."
Copy-Item -LiteralPath $donorBackup -Destination $target.Root -Force

Write-Host ""
Write-Host "After the target reboots, put the left half back into bootloader and flash the left firmware UF2."
