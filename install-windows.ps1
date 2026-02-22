[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repoUrl = "https://github.com/heabijay/crtcli"

function Get-WindowsTarget() {
    $arch = if ($env:PROCESSOR_ARCHITEW6432) {
        $env:PROCESSOR_ARCHITEW6432
    } else {
        $env:PROCESSOR_ARCHITECTURE
    }

    if (-not $arch) {
        return "x86_64-pc-windows-msvc"
        # throw "Cannot detect Windows architecture from PROCESSOR_ARCHITECTURE variables."
    }

    switch ($arch.ToUpperInvariant()) {
        "AMD64" { return "x86_64-pc-windows-msvc" }
        "ARM64" { return "aarch64-pc-windows-msvc" }
        # default { throw "Unsupported Windows architecture '$arch'. Supported: AMD64 and ARM64." }
        default { return "x86_64-pc-windows-msvc" }
    }
}

$target = Get-WindowsTarget
Write-Output "Detected target: $target"

$metadataUrl = if ($env:CRTCLI_INSTALL_METADATA_URL) {
    $env:CRTCLI_INSTALL_METADATA_URL
} else {
    "$repoUrl/releases/latest/download/release.json"
}

function Get-ArchiveNameFromTag([string] $tag) {
    return "crtcli-$tag-$target.zip"
}

function Resolve-LatestTag() {
    $request = [System.Net.HttpWebRequest]::Create("$repoUrl/releases/latest")
    $request.Method = "GET"
    $request.AllowAutoRedirect = $true
    $request.UserAgent = "crtcli-install-script"

    try {
        $response = $request.GetResponse()
    } catch [System.Net.WebException] {
        if ($_.Exception.Response) {
            $response = $_.Exception.Response
        } else {
            throw
        }
    }

    try {
        $effectiveUri = $response.ResponseUri.AbsoluteUri
    } finally {
        if ($response) {
            $response.Close()
        }
    }

    if ($effectiveUri -match '/tag/([^/?#]+)') {
        return $matches[1]
    }

    throw "Cannot resolve the latest crtcli release tag from $effectiveUri."
}

function Resolve-DownloadInfo() {
    if ($env:CRTCLI_INSTALL_VERSION_TAG) {
        $tag = $env:CRTCLI_INSTALL_VERSION_TAG
        $archiveName = Get-ArchiveNameFromTag $tag
        $downloadUrl = "$repoUrl/releases/download/$tag/$archiveName"

        return @{
            ArchiveName = $archiveName
            DownloadUrl = $downloadUrl
        }
    }

    try {
        $metadata = Invoke-RestMethod -Method Get -Uri $metadataUrl
        $asset = $metadata.assets.$target

        if ($asset -and $asset.download_url -and $asset.archive_name) {
            return @{
                ArchiveName = $asset.archive_name
                DownloadUrl = $asset.download_url
            }
        }
    } catch {
    }

    Write-Warning "Unable to use release metadata at $metadataUrl. Falling back to release tag discovery."

    $latestTag = Resolve-LatestTag
    $archiveName = Get-ArchiveNameFromTag $latestTag
    $downloadUrl = "$repoUrl/releases/download/$latestTag/$archiveName"

    return @{
        ArchiveName = $archiveName
        DownloadUrl = $downloadUrl
    }
}

$downloadInfo = Resolve-DownloadInfo
$destdir = "$env:LOCALAPPDATA\crtcli"
$zipfile = "$env:TEMP\$($downloadInfo.ArchiveName)"

Write-Output "Downloading: $($downloadInfo.ArchiveName)"
$webClient = New-Object System.Net.WebClient
$webClient.Headers.Add("user-agent", "crtcli-install-script")
try {
    $webClient.DownloadFile($downloadInfo.DownloadUrl, $zipfile)
} finally {
    $webClient.Dispose()
}

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
$userPath = if ($userPath) { $userPath } else { '' }
$machinePath = if ($machinePath) { $machinePath } else { '' }

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
