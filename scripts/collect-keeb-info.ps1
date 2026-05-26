param(
    [string]$OutDir = ".\keeb-info",
    [switch]$FullPnP
)

$ErrorActionPreference = "Continue"

function Write-Section {
    param([string]$Title)
    "`n==== $Title ====`n"
}

function Run-Command {
    param(
        [string]$Title,
        [scriptblock]$Command
    )

    Write-Section $Title
    try {
        & $Command | Out-String -Width 4096
    } catch {
        "ERROR: $($_.Exception.Message)`n"
    }
}

function Get-FileIfExists {
    param([string]$Path)

    if (Test-Path -LiteralPath $Path) {
        try {
            Get-Content -LiteralPath $Path -Raw
        } catch {
            "ERROR reading ${Path}: $($_.Exception.Message)"
        }
    }
}

$root = Resolve-Path -LiteralPath $PSScriptRoot\..
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$outPath = Join-Path $root $OutDir
New-Item -ItemType Directory -Force -Path $outPath | Out-Null
$report = Join-Path $outPath "keeb-info-$timestamp.txt"

$patterns = @(
    "keyboard",
    "hid",
    "usb",
    "nrf",
    "nordic",
    "adafruit",
    "uf2",
    "boot",
    "nice",
    "nano",
    "imprint",
    "zmk",
    "rmk",
    "cdc",
    "serial",
    "com[0-9]",
    "mass storage",
    "composite"
)
$patternRegex = ($patterns -join "|")

$sections = New-Object System.Collections.Generic.List[string]

$sections.Add((Run-Command "How to use this report" {
    @"
Run this script once with the keyboard in normal wired mode.
Then put the keyboard half into bootloader mode, run it again, and send both reports.

For many nRF52840 boards, bootloader mode appears as a removable UF2 drive containing INFO_UF2.TXT.
For split boards, test each half directly over USB if possible.
"@
}))

$sections.Add((Run-Command "Host" {
    [PSCustomObject]@{
        Time              = (Get-Date).ToString("o")
        User              = [Environment]::UserName
        Machine           = [Environment]::MachineName
        PowerShell        = $PSVersionTable.PSVersion.ToString()
        PSEdition         = $PSVersionTable.PSEdition
        OS                = (Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue).Caption
        OSVersion         = [Environment]::OSVersion.VersionString
        CurrentDirectory  = (Get-Location).Path
        ScriptRoot        = $PSScriptRoot
    } | Format-List
}))

$sections.Add((Run-Command "Filesystem drives" {
    Get-PSDrive -PSProvider FileSystem |
        Select-Object Name,Root,Description,Free,Used |
        Sort-Object Name |
        Format-Table -AutoSize
}))

$sections.Add((Run-Command "Removable drive hints and UF2 metadata" {
    $results = New-Object System.Collections.Generic.List[object]
    Get-PSDrive -PSProvider FileSystem | ForEach-Object {
        $driveRoot = $_.Root
        $infoUf2 = Join-Path $driveRoot "INFO_UF2.TXT"
        $index = Join-Path $driveRoot "INDEX.HTM"
        $current = [ordered]@{
            Drive     = $_.Name
            Root      = $driveRoot
            InfoUf2   = Test-Path -LiteralPath $infoUf2
            IndexHtm  = Test-Path -LiteralPath $index
            InfoText  = $null
        }
        if ($current.InfoUf2) {
            $current.InfoText = (Get-FileIfExists $infoUf2)
        }
        $results.Add([PSCustomObject]$current)
    }
    $results | Format-List
}))

$sections.Add((Run-Command "pnputil filtered connected devices" {
    $text = pnputil /enum-devices /connected 2>&1 | Out-String -Width 4096
    $blocks = $text -split "(`r?`n){2,}" | Where-Object { $_ -match $patternRegex }
    if ($blocks) {
        $blocks -join "`n`n"
    } else {
        "No matching connected PnP device blocks found. Re-run with -FullPnP if needed."
    }
}))

if ($FullPnP) {
    $sections.Add((Run-Command "pnputil all connected devices" {
        pnputil /enum-devices /connected 2>&1
    }))
}

$sections.Add((Run-Command "CIM PnP filtered devices" {
    Get-CimInstance Win32_PnPEntity -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -match $patternRegex -or
            $_.DeviceID -match "VID_|PID_|MI_|HID|USB|BTHLE|COM[0-9]"
        } |
        Select-Object PNPClass,Name,DeviceID,Manufacturer,Status |
        Sort-Object PNPClass,Name,DeviceID |
        Format-List
}))

$sections.Add((Run-Command "Serial ports" {
    Get-CimInstance Win32_SerialPort -ErrorAction SilentlyContinue |
        Select-Object Name,DeviceID,PNPDeviceID,Description,ProviderType,Status |
        Format-List
}))

$sections.Add((Run-Command "Keyboard class devices" {
    Get-CimInstance Win32_Keyboard -ErrorAction SilentlyContinue |
        Select-Object Name,Description,DeviceID,PNPDeviceID,Status |
        Format-List
}))

$sections.Add((Run-Command "USB controllers and hubs" {
    Get-CimInstance Win32_USBController -ErrorAction SilentlyContinue |
        Select-Object Name,DeviceID,PNPDeviceID,Status |
        Format-List
    Get-CimInstance Win32_USBHub -ErrorAction SilentlyContinue |
        Select-Object Name,DeviceID,PNPDeviceID,Status |
        Format-List
}))

$sections.Add((Run-Command "Toolchain availability" {
    $tools = @("cargo", "rustc", "rustup", "probe-rs", "cargo-make", "elf2uf2-rs", "cargo-hex-to-uf2")
    foreach ($tool in $tools) {
        $cmd = Get-Command $tool -ErrorAction SilentlyContinue
        if ($cmd) {
            "$tool => $($cmd.Source)"
            try { & $tool --version 2>&1 | Select-Object -First 3 } catch {}
        } else {
            "$tool => not found"
        }
        ""
    }
}))

$sections.Add((Run-Command "RMK nRF split files present" {
    $paths = @(
        "examples\use_config\nrf52840_ble_split\README.md",
        "examples\use_config\nrf52840_ble_split\keyboard.toml",
        "examples\use_config\nrf52840_ble_split\memory.x",
        "examples\use_config\nrf52840_ble_split\Makefile.toml",
        "examples\use_config\nrf52840_ble_split_direct_pin\README.md",
        "examples\use_config\nrf52840_ble_split_direct_pin\keyboard.toml",
        "examples\use_rust\nrf52840_ble_split\README.md",
        "docs\docs\main\docs\user_guide\flash_firmware.mdx"
    )
    foreach ($relative in $paths) {
        $path = Join-Path $root $relative
        if (Test-Path -LiteralPath $path) {
            "FOUND $relative"
        } else {
            "MISSING $relative"
        }
    }
}))

$content = $sections -join "`n"
Set-Content -LiteralPath $report -Value $content -Encoding UTF8

Write-Host "Wrote report: $report"
Write-Host ""
Write-Host "Next:"
Write-Host "1. Run this now in normal wired mode:"
Write-Host "   pwsh -ExecutionPolicy Bypass -File .\scripts\collect-keeb-info.ps1"
Write-Host "2. Put one half into bootloader mode, run it again, then send both report files."
Write-Host "3. If the filtered report misses the device, run:"
Write-Host "   pwsh -ExecutionPolicy Bypass -File .\scripts\collect-keeb-info.ps1 -FullPnP"
