# Draws the app icon: an original 64x64 pixel-art bust of Curly Brace (no game art is copied),
# scaled 16x to 1024x1024. Then run make_icons.ps1 -Source on the result for the bundle sizes.
#   powershell -NoProfile -ExecutionPolicy Bypass -File ios/make_curly_icon.ps1
param([string]$Out = "$PSScriptRoot\curly-icon-1024.png")

Add-Type -AssemblyName System.Drawing

$N = 64
function C($hex) { [System.Drawing.ColorTranslator]::FromHtml("#$hex") }
function InEllipse($x, $y, $cx, $cy, $rx, $ry) {
    $dx = ($x + 0.5 - $cx) / $rx; $dy = ($y + 0.5 - $cy) / $ry
    return ($dx * $dx + $dy * $dy) -le 1.0
}

$hair = C 'E2C200'; $hairShade = C 'B98C25'; $hairShine = C 'F6E27A'
$skin = C 'EEF3E2'; $skinShade = C 'A9BBCF'
$eye = C '1D4FE0'; $eyeLid = C '0B1F7A'; $white = C 'FFFFFF'
$blush = C 'F2A6AE'; $mouth = C '7A2838'
$top = C 'FF4E4A'; $topShade = C 'C8303A'
$phone = C '1E3A26'; $phoneRing = C '2F5E3A'; $phoneShine = C '6BCA67'
$antenna = C '4FA85A'; $antennaTip = C 'B4FF3C'
$outline = C '14182A'

# $null = background, filled in last.
$px = New-Object 'object[,]' $N, $N

for ($y = 0; $y -lt $N; $y++) {
    for ($x = 0; $x -lt $N; $x++) {
        $c = $null

        # Shoulders in her red top, darker on the shaded side.
        if (InEllipse $x $y 32 67 24 15) { $c = if ($x -ge 42) { $topShade } else { $top } }
        # Neck.
        if ($x -ge 29 -and $x -le 34 -and $y -ge 45 -and $y -le 53) { $c = $skinShade }
        # Hair: a golden mop with locks hanging beside her face, darker toward their tips.
        $locks = (InEllipse $x $y 12.5 42 5 11) -or (InEllipse $x $y 51.5 42 5 11)
        if ((InEllipse $x $y 32 28 23 19) -or $locks) {
            $c = if ($locks -and $y -ge 47) { $hairShade } else { $hair }
            # A curved shine across the crown.
            if ($y -lt 22 -and (InEllipse $x $y 29 23 17 12) -and -not (InEllipse $x $y 30 26 17 12)) { $c = $hairShine }
        }
        # Face, shaded along the jaw.
        if (InEllipse $x $y 32 36 14 13.5) {
            $c = if (InEllipse $x $y 31 34.5 13 12.5) { $skin } else { $skinShade }
            # Bangs: a straight fringe with three strands pointing down.
            $edge = 28
            foreach ($strand in @(@(23, 3), @(32, 2), @(41, 3))) {
                $edge = [Math]::Max($edge, 28 + $strand[1] - [Math]::Abs($x - $strand[0]))
            }
            if ($y -lt $edge) { $c = $hair }
        }
        # Eyes: lid, iris, highlight.
        foreach ($ex in 26, 38) {
            if (InEllipse $x $y $ex 37.5 2.7 4.3) {
                $c = if ($y -le 34) { $eyeLid } else { $eye }
                if (($x -eq $ex - 2 -or $x -eq $ex - 1) -and ($y -eq 35 -or $y -eq 36)) { $c = $white }
            }
        }
        if (($y -eq 42) -and ($x -in 21, 22, 42, 43)) { $c = $blush }
        if ((($y -eq 44) -and ($x -in 31, 32, 33)) -or (($y -eq 45) -and ($x -eq 32))) { $c = $mouth }

        # Headphone on her left (viewer's right), with its antenna.
        if (InEllipse $x $y 53 37 6 7) {
            $c = if (InEllipse $x $y 53 37 4 5) { $phoneRing } else { $phone }
            if (($x -eq 51 -or $x -eq 52) -and ($y -eq 34 -or $y -eq 35)) { $c = $phoneShine }
        }
        $px[$x, $y] = $c
    }
}

# Dark outline wherever the figure meets the background.
function Outline($px) {
    $edges = @()
    for ($y = 0; $y -lt $N; $y++) {
        for ($x = 0; $x -lt $N; $x++) {
            if ($null -eq $px[$x, $y]) { continue }
            foreach ($d in @(@(1, 0), @(-1, 0), @(0, 1), @(0, -1))) {
                $nx = $x + $d[0]; $ny = $y + $d[1]
                if ($nx -ge 0 -and $ny -ge 0 -and $nx -lt $N -and $ny -lt $N -and $null -eq $px[$nx, $ny]) { $edges += , @($x, $y); break }
            }
        }
    }
    return $edges
}
foreach ($e in (Outline $px)) { $px[$e[0], $e[1]] = $outline }

# The antenna goes on after the outline, so the thin stalk keeps its colour; it gets its own outline.
$stalk = New-Object 'object[,]' $N, $N
for ($y = 6; $y -le 31; $y++) {
    $x = [int][Math]::Round(55 + (31 - $y) * 3.0 / 22.0)
    foreach ($w in 0, 1) { if ($null -eq $px[($x + $w), $y]) { $stalk[($x + $w), $y] = $antenna } }
}
for ($y = 0; $y -lt $N; $y++) { for ($x = 0; $x -lt $N; $x++) { if (InEllipse $x $y 58.5 7 2.6 2.6) { $stalk[$x, $y] = $antennaTip } } }
for ($y = 0; $y -lt $N; $y++) {
    for ($x = 0; $x -lt $N; $x++) {
        if ($null -eq $stalk[$x, $y]) { continue }
        $px[$x, $y] = $stalk[$x, $y]
        foreach ($d in @(@(1, 0), @(-1, 0), @(0, 1), @(0, -1))) {
            $nx = $x + $d[0]; $ny = $y + $d[1]
            if ($nx -ge 0 -and $ny -ge 0 -and $nx -lt $N -and $ny -lt $N -and $null -eq $px[$nx, $ny] -and $null -eq $stalk[$nx, $ny]) { $px[$nx, $ny] = $outline }
        }
    }
}

# Night sky with a few stars, like the title screen.
$stars = @(@(6, 8), @(12, 3), @(47, 5), @(5, 26), @(60, 22), @(41, 13))
$bmp = New-Object System.Drawing.Bitmap $N, $N
for ($y = 0; $y -lt $N; $y++) {
    for ($x = 0; $x -lt $N; $x++) {
        $c = $px[$x, $y]
        if ($null -eq $c) {
            $t = $y / ($N - 1)
            $c = [System.Drawing.Color]::FromArgb(255, [int](32 - 14 * $t), [int](48 - 20 * $t), [int](88 - 38 * $t))
            foreach ($s in $stars) { if ($s[0] -eq $x -and $s[1] -eq $y) { $c = C 'DDE6FF' } }
        }
        $bmp.SetPixel($x, $y, $c)
    }
}

$big = New-Object System.Drawing.Bitmap 1024, 1024
$g = [System.Drawing.Graphics]::FromImage($big)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g.DrawImage($bmp, 0, 0, 1024, 1024)
$g.Dispose()
$big.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$big.Dispose(); $bmp.Dispose()
Write-Output "wrote $Out"
