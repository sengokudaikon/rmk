param(
    [switch]$IncludeRepeats,
    [string]$LogPath,
    [switch]$Check
)

$ErrorActionPreference = 'Stop'

$source = @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

public static class KeyboardHookLogger
{
    public delegate void KeyEventHandler(string eventName, int vkCode, int scanCode, int flags, long tick);
    public static event KeyEventHandler OnKeyEvent;

    private const int WH_KEYBOARD_LL = 13;
    private const int WM_KEYDOWN = 0x0100;
    private const int WM_KEYUP = 0x0101;
    private const int WM_SYSKEYDOWN = 0x0104;
    private const int WM_SYSKEYUP = 0x0105;

    private static LowLevelKeyboardProc _proc = HookCallback;
    private static IntPtr _hookID = IntPtr.Zero;

    public static void Start()
    {
        if (_hookID != IntPtr.Zero) return;
        _hookID = SetHook(_proc);
        if (_hookID == IntPtr.Zero)
        {
            throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        }
    }

    public static void Stop()
    {
        if (_hookID == IntPtr.Zero) return;
        UnhookWindowsHookEx(_hookID);
        _hookID = IntPtr.Zero;
    }

    private static IntPtr SetHook(LowLevelKeyboardProc proc)
    {
        using (Process curProcess = Process.GetCurrentProcess())
        using (ProcessModule curModule = curProcess.MainModule)
        {
            return SetWindowsHookEx(WH_KEYBOARD_LL, proc, GetModuleHandle(curModule.ModuleName), 0);
        }
    }

    private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    private struct KBDLLHOOKSTRUCT
    {
        public int vkCode;
        public int scanCode;
        public int flags;
        public int time;
        public IntPtr dwExtraInfo;
    }

    private static IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            int msg = wParam.ToInt32();
            string eventName = null;
            if (msg == WM_KEYDOWN) eventName = "down";
            else if (msg == WM_KEYUP) eventName = "up";
            else if (msg == WM_SYSKEYDOWN) eventName = "sys-down";
            else if (msg == WM_SYSKEYUP) eventName = "sys-up";

            if (eventName != null)
            {
                KBDLLHOOKSTRUCT data = Marshal.PtrToStructure<KBDLLHOOKSTRUCT>(lParam);
                KeyEventHandler handler = OnKeyEvent;
                if (handler != null)
                {
                    handler(eventName, data.vkCode, data.scanCode, data.flags, Stopwatch.GetTimestamp());
                }
            }
        }
        return CallNextHookEx(_hookID, nCode, wParam, lParam);
    }

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelKeyboardProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    private static extern IntPtr GetModuleHandle(string lpModuleName);
}
'@

Add-Type -TypeDefinition $source -ReferencedAssemblies System.Windows.Forms

if ($Check) {
    'Keyboard listener compiled OK.'
    exit 0
}

$seenDown = @{}
$started = Get-Date
$lastTick = 0L
$tickFrequency = [Diagnostics.Stopwatch]::Frequency
$writer = $null

if ($LogPath) {
    $resolvedLogPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($LogPath)
    $writer = [System.IO.StreamWriter]::new($resolvedLogPath, $true, [System.Text.Encoding]::UTF8)
}

function Format-KeyName {
    param([int]$VkCode)
    try {
        return ([System.Windows.Forms.Keys]$VkCode).ToString()
    } catch {
        return "VK_$VkCode"
    }
}

function Write-KeyLine {
    param(
        [string]$EventName,
        [int]$VkCode,
        [int]$ScanCode,
        [int]$Flags,
        [long]$Tick
    )

    if (-not $IncludeRepeats -and ($EventName -eq 'down' -or $EventName -eq 'sys-down')) {
        if ($seenDown.ContainsKey($VkCode)) {
            return
        }
        $seenDown[$VkCode] = $true
    } elseif ($EventName -eq 'up' -or $EventName -eq 'sys-up') {
        $seenDown.Remove($VkCode) | Out-Null
    }

    $elapsedMs = [math]::Round(((Get-Date) - $started).TotalMilliseconds)
    $deltaMs = if ($script:lastTick -eq 0) { 0 } else { [math]::Round((($Tick - $script:lastTick) * 1000.0) / $tickFrequency, 1) }
    $script:lastTick = $Tick

    $name = Format-KeyName $VkCode
    $line = '{0,8}ms +{1,6}ms {2,-8} vk=0x{3:X2} sc=0x{4:X2} flags=0x{5:X2} {6}' -f $elapsedMs, $deltaMs, $EventName, $VkCode, $ScanCode, $Flags, $name
    Write-Host $line
    if ($writer) {
        $writer.WriteLine($line)
        $writer.Flush()
    }
}

$handler = [KeyboardHookLogger+KeyEventHandler]{
    param($eventName, $vkCode, $scanCode, $flags, $tick)
    Write-KeyLine -EventName $eventName -VkCode $vkCode -ScanCode $scanCode -Flags $flags -Tick $tick
}

[KeyboardHookLogger]::add_OnKeyEvent($handler)

try {
    [KeyboardHookLogger]::Start()
    Write-Host 'Keyboard listener active. Press Ctrl+C in this window to stop.'
    Write-Host 'This logs HID-visible key events only; firmware-only BT/profile actions may not produce output.'
    while ($true) {
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 20
    }
}
finally {
    [KeyboardHookLogger]::remove_OnKeyEvent($handler)
    [KeyboardHookLogger]::Stop()
    if ($writer) {
        $writer.Dispose()
    }
}
