$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

try {
    if ([string]::IsNullOrWhiteSpace($env:AETHERIS_CAPTURE_EXECUTABLE) -or
        [string]::IsNullOrWhiteSpace($env:AETHERIS_CAPTURE_FIREWALL_RULE)) {
        throw 'Missing capture firewall configuration'
    }

    $parameters = @{
        PolicyStore = 'PersistentStore'
        Name = $env:AETHERIS_CAPTURE_FIREWALL_RULE
        Description = 'Reception TCP pour la capture reseau native Aetheris.'
        Program = $env:AETHERIS_CAPTURE_EXECUTABLE
        Direction = 'Inbound'
        Action = 'Allow'
        Enabled = 'True'
        Profile = 'Any'
        Protocol = 'TCP'
        LocalAddress = 'Any'
        RemoteAddress = 'Any'
        LocalPort = 'Any'
        RemotePort = 'Any'
        EdgeTraversalPolicy = 'Block'
    }
    $existing = Get-NetFirewallRule -PolicyStore PersistentStore -Name $parameters.Name -ErrorAction SilentlyContinue
    if ($env:AETHERIS_CAPTURE_FIREWALL_ACTION -eq 'remove') {
        if ($null -ne $existing) {
            Remove-NetFirewallRule -PolicyStore PersistentStore -Name $parameters.Name | Out-Null
        }
    } elseif ($null -eq $existing) {
        New-NetFirewallRule @parameters -DisplayName 'Aetheris - Capture TCP entrante' | Out-Null
    } else {
        Set-NetFirewallRule @parameters -NewDisplayName 'Aetheris - Capture TCP entrante' | Out-Null
    }
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
