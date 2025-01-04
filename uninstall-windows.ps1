$destdir = "$env:LOCALAPPDATA\crtcli"

Write-Output "Uninstalling crtcli..."

# Check if crtcli is installed
if (Test-Path -Path $destdir)
{
    Write-Output ""
    Write-Output "The following files and directories will be removed:"
    Write-Output "  $destdir"
    Write-Output ""

    if ($env:CRTCLI_INSTALL_CONFIRM_YES -ne 'true')
    {
        # Prompt user for confirmation before proceeding
        $continue = Read-Host -Prompt "Are you sure you want to uninstall crtcli and delete these files? (y/n)"
        if ($continue -ne 'y')
        {
            Write-Output ""
            Write-Output "Uninstallation aborted."
            return  # Exit the script
        }
    }

    Write-Output ""
    Write-Output "Removing crtcli installation from $destdir"

    # Stop any running processes that might be using crtcli files
    try {
        Stop-Process -Name "crtcli" -ErrorAction SilentlyContinue
    } catch {
        Write-Warning "Failed to stop crtcli process. Please ensure it's not running before proceeding."
    }

    # Remove the installation directory
    Remove-Item -Recurse -Force $destdir

    # Remove crtcli from the PATH environment variable (User)
    $userPath = [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::User)
    if ($userPath.ToLower().Contains($destdir.ToLower()))
    {
        Write-Output ""
        Write-Output "Removing $destdir from the User PATH variable."
        $newUserPath = ($userPath -split ';' | Where-Object { $_ -notlike "$destdir" }) -join ';'
        [System.Environment]::SetEnvironmentVariable('Path', $newUserPath, [System.EnvironmentVariableTarget]::User)
    }

    # Remove crtcli from the PATH environment variable (Machine) - Requires elevation
    $machinePath = [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::Machine)
    if ($machinePath.ToLower().Contains($destdir.ToLower()))
    {
        Write-Output ""
        Write-Output "Removing $destdir from the Machine PATH variable (requires elevation)."

        # Check if the script is running with administrator privileges
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object System.Security.Principal.WindowsPrincipal($identity)
        $adminRole = [System.Security.Principal.WindowsBuiltInRole]::Administrator

        if ($principal.IsInRole($adminRole)) {
            $newMachinePath = ($machinePath -split ';' | Where-Object { $_ -notlike "$destdir" }) -join ';'
            [System.Environment]::SetEnvironmentVariable('Path', $newMachinePath, [System.EnvironmentVariableTarget]::Machine)
        } else {
            Write-Warning "Administrator privileges are required to remove crtcli from the Machine PATH variable."
            Write-Output ""

            # Prompt user for elevation to remove from Machine PATH
            $elevate = Read-Host -Prompt "Do you want to elevate permissions to remove crtcli from the Machine PATH? (y/n)"
            if ($elevate -eq 'y')
            {
                # Re-run the script with elevation
                Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile", "-ExecutionPolicy Bypass", "-File", "`"$PSCommandPath`""
                return # Exit current instance
            } else {
                Write-Warning "crtcli was not removed from the Machine PATH."
            }
        }
    }

    # Refresh the current terminal's PATH
    Write-Output ""
    Write-Output "Refreshing current terminal's PATH."
    $Env:Path = [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::User) + ";" + [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::Machine)

    Write-Output ""
    Write-Output "crtcli has been uninstalled."
    Write-Output "For other terminals, restart them (or the entire IDE if they're within one)."
}
else
{
    Write-Output ""
    Write-Output "crtcli is not installed in the default location ($destdir)."
}

Write-Output ""
Write-Output "Uninstallation complete."