param(
    [string]$Title = "Claude Code",
    [string]$FallbackMessage = "",
    [string]$JsonInput = ""
)

# --- Win32 API ---
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}
"@

# --- 1. Find our terminal window and check focus ---
# Look for WindowsTerminal or mintty with a window handle
$terminalProc = Get-Process -Name 'WindowsTerminal' -ErrorAction SilentlyContinue |
    Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1

if (-not $terminalProc) {
    $terminalProc = Get-Process -Name 'mintty' -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
}

$terminalHwnd = if ($terminalProc) { $terminalProc.MainWindowHandle } else { [IntPtr]::Zero }

# Check if terminal is the foreground window — skip if focused
if ($terminalHwnd -ne [IntPtr]::Zero) {
    $fgHwnd = [Win32]::GetForegroundWindow()
    if ($fgHwnd -eq $terminalHwnd) { exit }
}

# --- 2. Parse JSON input from hook ---
$Project = ""
$Message = ""

if ($JsonInput) {
    try {
        $json = $JsonInput | ConvertFrom-Json
        if ($json.cwd) { $Project = ($json.cwd -split '[/\\]')[-1] }
        if ($json.message) { $Message = $json.message }
        elseif ($json.last_assistant_message) {
            $Message = $json.last_assistant_message
            if ($Message.Length -gt 200) { $Message = $Message.Substring(0, 200) + "..." }
        }
    } catch {}
}

if (-not $Message -and $FallbackMessage) { $Message = $FallbackMessage }

# --- 3. Build and show WPF toast ---
Add-Type -AssemblyName PresentationFramework
Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName WindowsBase

$rows = @('<RowDefinition Height="Auto"/>')
if ($Project) { $rows += '<RowDefinition Height="Auto"/>' }
if ($Message) { $rows += '<RowDefinition Height="Auto"/>' }
$rowDefs = $rows -join "`n                "

$eTitle = [System.Security.SecurityElement]::Escape($Title)
$eProject = [System.Security.SecurityElement]::Escape($Project)
$eMessage = [System.Security.SecurityElement]::Escape($Message)

$currentRow = 0
$contentBlocks = @()

$contentBlocks += @"
            <TextBlock Grid.Row="$currentRow" FontSize="16" FontWeight="SemiBold"
                       Foreground="#e8a849" Margin="0,0,0,6" TextWrapping="Wrap"
                       Text="$eTitle"/>
"@
$currentRow++

if ($Project) {
    $contentBlocks += @"
            <TextBlock Grid.Row="$currentRow" FontSize="13"
                       Foreground="#8888aa" Margin="0,0,0,6" TextWrapping="Wrap"
                       Text="$eProject"/>
"@
    $currentRow++
}

if ($Message) {
    $contentBlocks += @"
            <TextBlock Grid.Row="$currentRow" FontSize="14"
                       Foreground="#d0d0e0" Margin="0,4,0,0" TextWrapping="Wrap"
                       Text="$eMessage"/>
"@
}

$content = $contentBlocks -join "`n"

$xaml = @"
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        WindowStyle="None" AllowsTransparency="True" Background="Transparent"
        Topmost="True" ShowInTaskbar="False" SizeToContent="WidthAndHeight"
        WindowStartupLocation="Manual" MaxWidth="450" MinWidth="360"
        Cursor="Hand">
    <Border Background="#1a1a2e" CornerRadius="12" Padding="24,20"
            BorderBrush="#3a3a5c" BorderThickness="1" Name="ToastBorder">
        <Border.Effect>
            <DropShadowEffect BlurRadius="20" ShadowDepth="4" Opacity="0.5" Color="#000000"/>
        </Border.Effect>
        <Grid>
            <Grid.RowDefinitions>
                $rowDefs
            </Grid.RowDefinitions>
$content
        </Grid>
    </Border>
</Window>
"@

$reader = [System.Xml.XmlReader]::Create([System.IO.StringReader]::new($xaml))
$window = [System.Windows.Markup.XamlReader]::Load($reader)

# Click: bring terminal to foreground and close toast
$window.Add_MouseLeftButtonDown({
    if ($terminalHwnd -ne [IntPtr]::Zero) {
        [Win32]::SetForegroundWindow($terminalHwnd) | Out-Null
    }
    $window.Close()
})

# Hover effect
$border = $window.FindName("ToastBorder")
$window.Add_MouseEnter({
    $border.BorderBrush = [System.Windows.Media.BrushConverter]::new().ConvertFrom("#5a5a8c")
})
$window.Add_MouseLeave({
    $border.BorderBrush = [System.Windows.Media.BrushConverter]::new().ConvertFrom("#3a3a5c")
})

# Position bottom-right above taskbar
$screen = [System.Windows.SystemParameters]::WorkArea
$window.Left = $screen.Right - 470
$window.Top = $screen.Bottom - 180

# Fade in
$window.Opacity = 0
$fadeIn = New-Object System.Windows.Media.Animation.DoubleAnimation(0, 1, (New-Object System.TimeSpan(0,0,0,0,300)))
$window.BeginAnimation([System.Windows.Window]::OpacityProperty, $fadeIn)

# Auto-close after 5 seconds with fade out
$timer = New-Object System.Windows.Threading.DispatcherTimer
$timer.Interval = New-Object System.TimeSpan(0,0,5)
$timer.Add_Tick({
    $fadeOut = New-Object System.Windows.Media.Animation.DoubleAnimation(1, 0, (New-Object System.TimeSpan(0,0,0,0,400)))
    $fadeOut.Add_Completed({ $window.Close() })
    $window.BeginAnimation([System.Windows.Window]::OpacityProperty, $fadeOut)
    $timer.Stop()
})
$timer.Start()

$window.ShowDialog() | Out-Null
