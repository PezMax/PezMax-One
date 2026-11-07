# uiwalk —— 用 UI Automation 驱动打包后的 UWP 客户端走一遍流程
#
# 为什么要有它：`dotnet test` 测的是 Core 的判定，`tools/apiwalk` 测的是协议，
# 但「点了有没有反应」只有真的点才知道——2026-11-07 就是靠它发现文件树点了不响应。
#
# 前提：客户端已安装并对本机地址加了环回豁免（见 docs/ENVIRONMENT.md）。
# 用法：
#   . .\windows\tools\uiwalk\uiwalk.ps1
#   Set-Text -Name '用户名' -Text 'demo'
#   Click-El -Name '登录'
#   Shot -Path "$env:TEMP\home.png"
#
# 注意：
# - 元素按中文名字找，界面文案改了它就得跟着改（这是故意的：名字就是用户看到的字）。
# - 覆盖层下面那一层还在 UIA 树里，同名元素会有两个，所以先排除屏幕外的、再取最后一个。
# - 它不写仓库里的任何文件，截图落在你给的路径。

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing

if (-not ('Win.Uia' -as [type])) {
    Add-Type -Namespace Win -Name Uia -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
'@
}

if (-not ('Win.Mouse' -as [type])) {
    Add-Type -Namespace Win -Name Mouse -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
[DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, System.UIntPtr e);
'@
}

# 窗口标题就是应用名；改了 App 的显示名这里要跟着改。
$script:AppWindowName = 'PezMax One'

function Get-AppWin {
    $root = [System.Windows.Automation.AutomationElement]::RootElement
    $cond = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::NameProperty, $script:AppWindowName)
    $win = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $cond)
    if (-not $win) { throw "找不到窗口「$($script:AppWindowName)」——客户端没开？" }
    # 置顶再截图，否则抓到的是后面那个窗口。
    [Win.Uia]::SetWindowPos([IntPtr]$win.Current.NativeWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x43) | Out-Null
    $win
}

function Find-El {
    param([string]$Name, [string]$Type)

    $win = Get-AppWin
    $conds = @()
    if ($Name) {
        $conds += New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::NameProperty, $Name)
    }
    if ($Type) {
        $conds += New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::$Type)
    }
    $cond = if ($conds.Count -eq 1) { $conds[0] } else { New-Object System.Windows.Automation.AndCondition($conds) }

    $hits = @($win.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond) |
        Where-Object { -not $_.Current.IsOffscreen })
    if ($hits.Count -eq 0) {
        $hits = @($win.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond))
    }
    if ($hits.Count -eq 0) { return $null }
    $hits[$hits.Count - 1]
}

function Set-Text {
    param([string]$Name, [string]$Text)
    $el = Find-El -Name $Name -Type Edit
    if (-not $el) { throw "找不到输入框「$Name」" }
    $el.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($Text)
    "  已填 $Name = $Text"
}

function Click-El {
    param([string]$Name, [string]$Type = 'Button')
    $el = Find-El -Name $Name -Type $Type
    if (-not $el) { throw "找不到「$Name」($Type)" }
    $el.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    "  已点 $Name"
}

function Select-Item {
    param([string]$Name)
    $el = Find-El -Name $Name -Type ListItem
    if (-not $el) { throw "找不到列表项「$Name」" }
    $el.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select()
    "  已选 $Name"
}

function Expand-Tree {
    param([string]$Name)
    $el = Find-El -Name $Name -Type TreeItem
    if (-not $el) { throw "找不到树节点「$Name」" }
    $el.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern).Expand()
    "  已展开 $Name"
}

# TreeView 的 ItemInvoked 只认真点击（UIA 的 Select 不触发它），所以必须点坐标。
function Click-Real {
    param([string]$Name, [string]$Type = 'ListItem')
    $el = Find-El -Name $Name -Type $Type
    if (-not $el) { throw "找不到「$Name」($Type)" }
    $r = $el.Current.BoundingRectangle
    $x = [int]($r.X + $r.Width / 2)
    $y = [int]($r.Y + $r.Height / 2)
    [Win.Mouse]::SetCursorPos($x, $y) | Out-Null
    Start-Sleep -Milliseconds 150
    [Win.Mouse]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 80
    [Win.Mouse]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    "  已点击 $Name @ ($x,$y)"
}

function Shot {
    param([string]$Path)
    Start-Sleep -Milliseconds 700
    $win = Get-AppWin
    $r = $win.Current.BoundingRectangle
    $bmp = New-Object System.Drawing.Bitmap([int]$r.Width, [int]$r.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.X, [int]$r.Y, 0, 0, $bmp.Size)
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
    "  截图 -> $Path"
}

# 看树用：把界面上每个有名字的元素连类型一起打出来。
function Dump-Ui {
    $win = Get-AppWin
    $all = $win.FindAll([System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.Condition]::TrueCondition)
    foreach ($el in $all) {
        $name = $el.Current.Name
        if (-not $name) { continue }
        $type = $el.Current.ControlType.ProgrammaticName -replace 'ControlType\.', ''
        $value = ''
        try { $value = $el.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value } catch { }
        "  $type | $name | $value"
    }
}
