# Builds the iOS app icons from a 1024x1024 source, by default the Curly icon from make_curly_icon.ps1.
# A source with see-through rounded corners (like res/macos/512x512@2x.png) is cropped to its opaque
# square and the corners filled with its background colour (iOS rounds the corners itself).
# Writes every size the bundle needs.
#   powershell -NoProfile -ExecutionPolicy Bypass -File ios/make_icons.ps1
param(
    [string]$Source = "$PSScriptRoot\curly-icon-1024.png",
    [string]$OutDir = "$PSScriptRoot\icons"
)

Add-Type -AssemblyName System.Drawing

$src = New-Object System.Drawing.Bitmap (Resolve-Path $Source).Path
$w = $src.Width; $h = $src.Height

# Bounds of the fully opaque rounded square (the drop shadow around it is partly transparent).
$minX = $w; $minY = $h; $maxX = 0; $maxY = 0
for ($y = 0; $y -lt $h; $y += 2) {
    for ($x = 0; $x -lt $w; $x += 2) {
        if ($src.GetPixel($x, $y).A -ge 250) {
            if ($x -lt $minX) { $minX = $x }; if ($x -gt $maxX) { $maxX = $x }
            if ($y -lt $minY) { $minY = $y }; if ($y -gt $maxY) { $maxY = $y }
        }
    }
}
$side = [Math]::Min($maxX - $minX, $maxY - $minY)
$bg = $src.GetPixel([int](($minX + $maxX) / 2), $minY + 12)
Write-Output "opaque square: x $minX-$maxX, y $minY-$maxY; background $bg"

New-Item -ItemType Directory -Force $OutDir | Out-Null
$sizes = [ordered]@{
    "AppIcon60x60@2x.png"          = 120
    "AppIcon60x60@3x.png"          = 180
    "AppIcon76x76@2x~ipad.png"     = 152
    "AppIcon83.5x83.5@2x~ipad.png" = 167
    "AppIcon-1024.png"             = 1024
}
foreach ($name in $sizes.Keys) {
    $n = $sizes[$name]
    $dst = New-Object System.Drawing.Bitmap $n, $n
    $g = [System.Drawing.Graphics]::FromImage($dst)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::FromArgb(255, $bg.R, $bg.G, $bg.B))
    $g.DrawImage($src, (New-Object System.Drawing.Rectangle 0, 0, $n, $n), $minX, $minY, $side, $side, [System.Drawing.GraphicsUnit]::Pixel)
    $g.Dispose()
    $dst.Save((Join-Path $OutDir $name), [System.Drawing.Imaging.ImageFormat]::Png)
    $dst.Dispose()
    Write-Output "$name ${n}x$n"
}
$src.Dispose()
