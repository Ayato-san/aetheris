# Run with Windows PowerShell. All firewall cmdlets are replaced by test doubles;
# this test never reads or changes the machine's firewall rules.
param([string]$Case)
$ErrorActionPreference = 'Stop'
$testPath = $PSCommandPath
if (-not $testPath) { $testPath = $env:AETHERIS_CAPTURE_TEST_SCRIPT }

if (-not $Case) {
    $powershell = (Get-Process -Id $PID).Path
    $env:AETHERIS_CAPTURE_TEST_SCRIPT = $testPath
    foreach ($scenario in @('create', 'update', 'denied', 'missing', 'remove', 'remove-missing')) {
        $env:AETHERIS_CAPTURE_TEST_CASE = $scenario
        $ErrorActionPreference = 'Continue'
        $result = & $powershell -NoProfile -NonInteractive -Command '& ([scriptblock]::Create([IO.File]::ReadAllText($env:AETHERIS_CAPTURE_TEST_SCRIPT))) -Case $env:AETHERIS_CAPTURE_TEST_CASE' 2>&1
        $ErrorActionPreference = 'Stop'
        $expected = if ($scenario -in @('denied', 'missing')) { 1 } else { 0 }
        if ($LASTEXITCODE -ne $expected) {
            throw "${scenario}: unexpected exit code $LASTEXITCODE : $result"
        }
        if ($scenario -eq 'denied' -and "$result" -notmatch 'TEST_ACCESS_DENIED') {
            throw "${scenario}: failure was not propagated: $result"
        }
        if ($scenario -eq 'missing' -and "$result" -notmatch 'Missing capture firewall configuration') {
            throw "${scenario}: missing executable was not rejected: $result"
        }
        Write-Output "PASS $scenario"
    }
    exit 0
}

$env:AETHERIS_CAPTURE_EXECUTABLE = 'C:\Aetheris & tests\l''ete $(ignored)\aetheris.exe'
$env:AETHERIS_CAPTURE_FIREWALL_RULE = 'Aetheris.DebugCapture.TCP.test'
$env:AETHERIS_CAPTURE_FIREWALL_ACTION = if ($Case -like 'remove*') { 'remove' } else { 'configure' }
$script:created = 0
$script:updated = 0
$script:removed = 0
$script:parameters = @{}

function Read-TestParameters($arguments) {
    for ($index = 0; $index -lt $arguments.Count; $index += 2) {
        $script:parameters[$arguments[$index].TrimStart('-').TrimEnd(':')] = $arguments[$index + 1]
    }
}
function Get-NetFirewallRule {
    if ($Case -in @('update', 'remove')) { return [pscustomobject]@{ Name = $env:AETHERIS_CAPTURE_FIREWALL_RULE } }
    return $null
}
function New-NetFirewallRule {
    if ($Case -eq 'denied') { throw 'TEST_ACCESS_DENIED' }
    $script:created++
    Read-TestParameters $args
}
function Set-NetFirewallRule {
    $script:updated++
    Read-TestParameters $args
}
function Remove-NetFirewallRule {
    $script:removed++
    Read-TestParameters $args
}

if ($Case -eq 'missing') { $env:AETHERIS_CAPTURE_EXECUTABLE = '' }
$configuration = Join-Path (Split-Path $testPath) '../src/capture/windows/firewall.ps1'
& ([scriptblock]::Create([IO.File]::ReadAllText($configuration)))

if ($Case -like 'remove*') {
    $expectedRemoved = if ($Case -eq 'remove') { 1 } else { 0 }
    if ($created -ne 0 -or $updated -ne 0 -or $removed -ne $expectedRemoved -or
        ($removed -ne 0 -and $parameters.Name -cne $env:AETHERIS_CAPTURE_FIREWALL_RULE)) {
        throw 'Uninstall must remove only the service rule and tolerate an absent rule'
    }
    exit 0
}

if (($Case -eq 'create' -and ($created -ne 1 -or $updated -ne 0)) -or
    ($Case -eq 'update' -and ($updated -ne 1 -or $created -ne 0))) {
    throw 'The rule must be created once or updated without adding a duplicate'
}
if ($parameters.Program -cne $env:AETHERIS_CAPTURE_EXECUTABLE -or
    $parameters.Name -cne $env:AETHERIS_CAPTURE_FIREWALL_RULE -or
    $parameters.PolicyStore -ne 'PersistentStore' -or
    $parameters.Direction -ne 'Inbound' -or
    $parameters.Protocol -ne 'TCP' -or
    $parameters.Action -ne 'Allow' -or
    $parameters.Enabled -ne 'True' -or
    $parameters.EdgeTraversalPolicy -ne 'Block') {
    throw "The permission must remain limited to the executable and inbound TCP: $($parameters | ConvertTo-Json -Compress)"
}
