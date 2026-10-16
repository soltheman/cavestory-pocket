# Makes a personal copy of the IPA whose home-screen icon is Curly's portrait, taken from YOUR
# Cave Story+ files. The portrait is Nicalis's art, so it never goes in this repo or a release:
# run this locally on a downloaded IPA and sideload the result yourself.
#   powershell -NoProfile -ExecutionPolicy Bypass -File ios/curly_icon_ipa.ps1 -Ipa dist/CaveStoryPocket.ipa
param(
    [Parameter(Mandatory = $true)][string]$Ipa,
    # The game's data folder (the one copied to the phone).
    [string]$Data = "C:\Program Files\Epic Games\CaveStoryPlus\data",
    # Defaults to <ipa name>-curly.ipa next to the input.
    [string]$Out = ""
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$Ipa = (Resolve-Path $Ipa).Path
if (-not $Out) { $Out = [IO.Path]::ChangeExtension($Ipa, $null).TrimEnd('.') + "-curly.ipa" }

# Face.bmp is a 6x5 grid of portraits (96px in Cave Story+, 48px in freeware).
# Curly's happy portrait is column 1, row 3.
$facePath = Join-Path $Data "base\Face.bmp"
if (-not (Test-Path $facePath)) { $facePath = Join-Path $Data "Face.bmp" }
$faces = [System.Drawing.Bitmap]::FromFile((Resolve-Path $facePath).Path)
$cell = [int]($faces.Width / 6)

# Blow the portrait up with hard pixel edges first, so the small icons stay sharp.
$big = New-Object System.Drawing.Bitmap 1024, 1024
$g = [System.Drawing.Graphics]::FromImage($big)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g.DrawImage($faces, (New-Object System.Drawing.Rectangle 0, 0, 1024, 1024), $cell * 1, $cell * 3, $cell, $cell, [System.Drawing.GraphicsUnit]::Pixel)
$g.Dispose()
$faces.Dispose()

$sizes = [ordered]@{
    "AppIcon60x60@2x.png"          = 120
    "AppIcon60x60@3x.png"          = 180
    "AppIcon76x76@2x~ipad.png"     = 152
    "AppIcon83.5x83.5@2x~ipad.png" = 167
}

Copy-Item $Ipa $Out -Force
$zip = [System.IO.Compression.ZipFile]::Open($Out, [System.IO.Compression.ZipArchiveMode]::Update)
try {
    $plist = $zip.Entries | Where-Object { $_.FullName -match '^Payload/[^/]+\.app/Info\.plist$' } | Select-Object -First 1
    if (-not $plist) { throw "No Payload/*.app/Info.plist in $Ipa" }
    $app = $plist.FullName.Substring(0, $plist.FullName.Length - "Info.plist".Length)

    foreach ($name in $sizes.Keys) {
        $n = $sizes[$name]
        $icon = New-Object System.Drawing.Bitmap $n, $n
        $g = [System.Drawing.Graphics]::FromImage($icon)
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $g.DrawImage($big, 0, 0, $n, $n)
        $g.Dispose()

        $old = $zip.GetEntry("$app$name")
        if ($old) { $old.Delete() }
        $entry = $zip.CreateEntry("$app$name")
        $entry.ExternalAttributes = 0x81A4 -shl 16  # regular file, rw-r--r--
        $stream = $entry.Open()
        $icon.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
        $stream.Dispose()
        $icon.Dispose()
        Write-Output "icon $name ${n}x$n"
    }
} finally {
    $zip.Dispose()
    $big.Dispose()
}

# .NET rewrites every entry's "made by" host as MS-DOS, and unzip tools then ignore the Unix modes,
# so the app binary would lose its executable bit. Mark the entries as Unix again in the central directory.
$bytes = [IO.File]::ReadAllBytes($Out)
$eocd = -1
for ($i = $bytes.Length - 22; $i -ge [Math]::Max(0, $bytes.Length - 65557); $i--) {
    if ([BitConverter]::ToUInt32($bytes, $i) -eq 0x06054b50) { $eocd = $i; break }
}
if ($eocd -lt 0) { throw "No end-of-central-directory record in $Out" }
$count = [BitConverter]::ToUInt16($bytes, $eocd + 10)
$pos = [BitConverter]::ToUInt32($bytes, $eocd + 16)
for ($k = 0; $k -lt $count; $k++) {
    if ([BitConverter]::ToUInt32($bytes, $pos) -ne 0x02014b50) { throw "Bad central directory entry $k in $Out" }
    $bytes[$pos + 5] = 3  # host: Unix
    $pos += 46 + [BitConverter]::ToUInt16($bytes, $pos + 28) + [BitConverter]::ToUInt16($bytes, $pos + 30) + [BitConverter]::ToUInt16($bytes, $pos + 32)
}
[IO.File]::WriteAllBytes($Out, $bytes)

# The app binary must still be executable after the rewrite.
$check = [System.IO.Compression.ZipFile]::OpenRead($Out)
try {
    foreach ($e in $check.Entries | Where-Object { $_.FullName -match '^Payload/[^/]+\.app/[^/.]+$' }) {
        $mode = ($e.ExternalAttributes -shr 16) -band 0xFFFF
        Write-Output ("binary {0} mode {1}" -f $e.FullName, [Convert]::ToString($mode, 8))
        if (($mode -band 0x49) -eq 0) { throw "$($e.FullName) lost its executable bit" }
    }
} finally {
    $check.Dispose()
}
Write-Output "wrote $Out"
