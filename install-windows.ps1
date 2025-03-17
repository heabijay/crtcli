## Thanks to another project author, used as reference (/boilerplate):
## https://github.com/ducaale/xh/blob/master/install.sh

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ProgressPreference = 'SilentlyContinue'

if ($env:CRTCLI_INSTALL_VERSION_TAG) {
    $releaseUrl = "https://api.github.com/repos/heabijay/crtcli/releases/tags/$($env:CRTCLI_INSTALL_VERSION_TAG)"
} else {
    $releaseUrl = "https://api.github.com/repos/heabijay/crtcli/releases/latest"
}

$release = Invoke-RestMethod -Method Get -Uri $releaseUrl
$asset = $release.assets | Where-Object name -like *x86_64-pc-windows*.zip
$destdir = "$env:LOCALAPPDATA\crtcli"
$zipfile = "$env:TEMP\$( $asset.name )"

Write-Output "Downloading: $( $asset.name )"
Invoke-RestMethod -Method Get -Uri $asset.browser_download_url -OutFile $zipfile

# Check if an older version of crtcli exists in '$destdir', if yes, then delete it, if not then download latest zip to extract from
if (Test-Path -Path $destdir)
{
    Write-Output ""
    Write-Output "Removing previous installation of crtcli from $destdir"
    Remove-Item -r -fo $destdir/*
}

# Create dir for result of extraction
New-Item -ItemType Directory -Path $destdir -Force | Out-Null

# Decompress the zip file to the destination directory
Add-Type -Assembly System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::OpenRead($zipfile)
$entries = $zip.Entries #| Where-Object { $_.FullName -like '*.exe' }
$entries | ForEach-Object { [IO.Compression.ZipFileExtensions]::ExtractToFile($_, $destdir + "\" + $_.Name) }

# Free the zipfile
$zip.Dispose()
Remove-Item -Path $zipfile

Write-Output ""
Invoke-Expression -Command "$destdir\crtcli.exe --version" -OutVariable crtcliVersion
Write-Output ""

# Inform user where the executables have been put
Write-Output "$( $crtcliVersion ) has been installed to:`n * $destdir\crtcli.exe"

# Make sure destdir is in the path
$userPath = [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::User)
$machinePath = [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::Machine)

# If userPath AND machinePath both do not contain crtcli dir, then add it to user path
if (!($userPath.ToLower().Contains($destdir.ToLower())) -and !($machinePath.ToLower().Contains($destdir.ToLower())))
{
    # Update userPath
    $userPath = $userPath.Trim(";") + ";$destdir"

    # Modify PATH for new windows
    Write-Output "`nAdding $destdir directory to the PATH variable."
    [System.Environment]::SetEnvironmentVariable('Path', $userPath, [System.EnvironmentVariableTarget]::User)

    # Modify PATH for current terminal
    Write-Output "`nRefreshing current terminal's PATH for you."
    $Env:Path = $Env:Path.Trim(";") + ";$destdir"

    # Instruct how to modify PATH for other open terminals
    Write-Output "`nFor other terminals, restart them (or the entire IDE if they're within one).`n"
}