# spec/tools/probe.ps1 —— 只读契约探针
#
# 用途：拿真实（或本地起的）PezMax-Go 核对 `spec/api/endpoints.md` 里写下的形状。
# **只发 GET、不写任何数据**，所以不需要账号；带 -Token 时顺带能核对「可选会话」的端点。
#
#   pwsh -File spec/tools/probe.ps1 -BaseUrl http://127.0.0.1:9033
#   pwsh -File spec/tools/probe.ps1 -BaseUrl https://example.com -Token <opaque>
#
# 输出是给夹具补 `source` 用的：状态码、耗时、顶层键、数组长度与首元素键。
# 发现形状与 endpoints.md 不一致时，先改 spec，再改实现。

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BaseUrl,

    [string]$Token,

    [int]$TimeoutSeconds = 20
)

$ErrorActionPreference = 'Stop'
$BaseUrl = $BaseUrl.TrimEnd('/')

$endpoints = @(
    @{ Name = '文件列表（匿名口径）'; Path = '/datum/file/list?pageNum=1&pageSize=2' },
    @{ Name = '学科及计数'; Path = '/datum/file/subjects' },
    @{ Name = '学校列表'; Path = '/datum/file/schools' },
    @{ Name = '树'; Path = '/datum/file/tree' },
    @{ Name = '搜索'; Path = '/datum/file/search?keyword=%E6%95%B0%E5%AD%A6' },
    @{ Name = '电子书列表'; Path = '/datum/ebook/list?pageNum=1&pageSize=2' },
    @{ Name = '电子书学科'; Path = '/datum/ebook/subjects' },
    @{ Name = '上传排行'; Path = '/datum/user/rank' }
)

$headers = @{ Accept = 'application/json' }
if ($Token) { $headers['Authorization'] = "Bearer $Token" }

function Format-Shape($value, [int]$depth = 0) {
    if ($null -eq $value) { return 'null' }
    if ($value -is [string]) { return 'string' }
    if ($value -is [bool]) { return 'bool' }
    if ($value -is [ValueType]) { return 'number' }
    if ($value -is [System.Collections.IEnumerable] -and $value -isnot [string]) {
        $items = @($value)
        if ($items.Count -eq 0) { return '[]' }
        $first = $items[0]
        $inner = if ($first -is [string] -or $first -is [ValueType]) { 'scalar' } else { (Format-Shape $first ($depth + 1)) }
        return "array[$($items.Count)] of $inner"
    }
    if ($value -is [System.Management.Automation.PSCustomObject]) {
        $keys = $value.PSObject.Properties.Name
        return 'obj{' + ($keys -join ',') + '}'
    }
    return 'unknown'
}

Write-Host ""
Write-Host "PezMax-Go 契约探针" -ForegroundColor Cyan
Write-Host "  base: $BaseUrl"
Write-Host "  会话: $(if ($Token) { '带 token（可选会话端点会用上）' } else { '匿名' })"
Write-Host ("  只读：" + $endpoints.Count + " 个 GET，不写任何数据")
Write-Host ""

$failed = 0
foreach ($endpoint in $endpoints) {
    $uri = $BaseUrl + $endpoint.Path
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $response = Invoke-WebRequest -Uri $uri -Headers $headers -TimeoutSec $TimeoutSeconds -UseBasicParsing
        $sw.Stop()
        $body = $response.Content
        $status = [int]$response.StatusCode
        $contentType = $response.Headers['Content-Type']
    }
    catch {
        $sw.Stop()
        $status = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { 0 }
        $body = $_.Exception.Message
        $contentType = ''
        if ($status -ne 0) { $failed++ }
    }

    Write-Host ("[{0,3}] {1,5}ms  {2}" -f $status, $sw.ElapsedMilliseconds, $endpoint.Name) -ForegroundColor $(if ($status -ge 200 -and $status -lt 300) { 'Green' } else { 'Yellow' })
    Write-Host ("        {0}" -f $endpoint.Path) -ForegroundColor DarkGray

    if ($status -lt 200 -or $status -ge 300) {
        Write-Host ("        响应：{0}" -f ($body -replace "\s+", ' ').Substring(0, [Math]::Min(160, ($body -replace "\s+", ' ').Length))) -ForegroundColor DarkYellow
        Write-Host ""
        continue
    }

    try {
        $json = $body | ConvertFrom-Json
    }
    catch {
        Write-Host ("        不是 JSON（Content-Type={0}）：{1}" -f $contentType, $body.Substring(0, [Math]::Min(80, $body.Length))) -ForegroundColor DarkYellow
        Write-Host ""
        continue
    }

    $top = $json.PSObject.Properties.Name -join ','
    Write-Host ("        顶层：{0}" -f $top) -ForegroundColor DarkGray
    foreach ($key in @('code', 'msg', 'total', 'hash', 'unchanged')) {
        if ($json.PSObject.Properties.Name -contains $key) {
            Write-Host ("          {0} = {1}" -f $key, ($json.$key -join ',')) -ForegroundColor DarkGray
        }
    }

    foreach ($key in @('data', 'rows')) {
        if ($json.PSObject.Properties.Name -contains $key -and $null -ne $json.$key) {
            Write-Host ("          {0}: {1}" -f $key, (Format-Shape $json.$key)) -ForegroundColor DarkGray
        }
    }

    Write-Host ""
}

if ($failed -gt 0) {
    Write-Host ("有 {0} 个端点连不上（非 2xx）。先确认 base、端口与 backend 是否在跑。" -f $failed) -ForegroundColor Yellow
    exit 1
}

Write-Host "全部端点返回 2xx。形状与 spec/api/endpoints.md 不一致时，先改 spec 再改实现。" -ForegroundColor Green
