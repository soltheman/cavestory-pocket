# Draws the touch-control art in src/data/builtin (all white, so the chosen button colour tints it):
#   touch_icons.png  128x320, 32px cells. Rows 0-3: doukutsu-rs's touch.png icons recoloured white.
#                    Rows 4-7: the same layout in a bold filled style. Row 8: weapon switching
#                    (classic previous, classic next, bold previous, bold next). Row 9: map
#                    (classic, bold), then the strafe lock padlock (classic, bold).
#   touch_plate.png   96x96, 48px cells: round backing, solid round, square backing, solid square.
#   powershell -NoProfile -ExecutionPolicy Bypass -File ios/make_touch_art.ps1
param([string]$Dir = "$PSScriptRoot\..\src\data\builtin")

Add-Type -AssemblyName System.Drawing
$Dir = (Resolve-Path $Dir).Path

function Sign($x1, $y1, $x2, $y2, $x3, $y3) { ($x1 - $x3) * ($y2 - $y3) - ($x2 - $x3) * ($y1 - $y3) }

# Shapes: @('t', x1, y1, x2, y2, x3, y3) triangle, @('r', x0, y0, x1, y1) inclusive rectangle.
function InShape($x, $y, $s) {
    $px = $x + 0.5; $py = $y + 0.5
    if ($s[0] -eq 'r') { return ($x -ge $s[1] -and $x -le $s[3] -and $y -ge $s[2] -and $y -le $s[4]) }
    $d1 = Sign $px $py $s[1] $s[2] $s[3] $s[4]
    $d2 = Sign $px $py $s[3] $s[4] $s[5] $s[6]
    $d3 = Sign $px $py $s[5] $s[6] $s[1] $s[2]
    $neg = ($d1 -lt 0) -or ($d2 -lt 0) -or ($d3 -lt 0)
    $pos = ($d1 -gt 0) -or ($d2 -gt 0) -or ($d3 -gt 0)
    return -not ($neg -and $pos)
}

function Mask($icon) {
    $m = New-Object 'bool[,]' 32, 32
    for ($y = 0; $y -lt 32; $y++) {
        for ($x = 0; $x -lt 32; $x++) {
            $in = $false
            foreach ($s in $icon.add) { if (InShape $x $y $s) { $in = $true; break } }
            if ($in) { foreach ($s in $icon.cut) { if (InShape $x $y $s) { $in = $false; break } } }
            $m[$x, $y] = $in
        }
    }
    return , $m
}

function IsSet($m, $x, $y) { $x -ge 0 -and $y -ge 0 -and $x -lt 32 -and $y -lt 32 -and $m[$x, $y] }

# Bold: solid white with a dark edge around it.
function DrawBold($bmp, $ox, $oy, $m) {
    for ($y = 0; $y -lt 32; $y++) {
        for ($x = 0; $x -lt 32; $x++) {
            if ($m[$x, $y]) { $bmp.SetPixel($ox + $x, $oy + $y, [System.Drawing.Color]::White); continue }
            $edge = $false
            foreach ($d in @(@(1, 0), @(-1, 0), @(0, 1), @(0, -1), @(1, 1), @(-1, -1), @(1, -1), @(-1, 1))) {
                if (IsSet $m ($x + $d[0]) ($y + $d[1])) { $edge = $true; break }
            }
            if ($edge) { $bmp.SetPixel($ox + $x, $oy + $y, [System.Drawing.Color]::FromArgb(170, 0, 0, 0)) }
        }
    }
}

# Classic: the shape's 2px outline with a grey shadow below-right, like doukutsu-rs's line icons.
function DrawClassic($bmp, $ox, $oy, $m) {
    $line = New-Object 'bool[,]' 32, 32
    for ($y = 0; $y -lt 32; $y++) {
        for ($x = 0; $x -lt 32; $x++) {
            if (-not $m[$x, $y]) { continue }
            foreach ($d in @(@(2, 0), @(-2, 0), @(0, 2), @(0, -2), @(1, 0), @(-1, 0), @(0, 1), @(0, -1))) {
                if (-not (IsSet $m ($x + $d[0]) ($y + $d[1]))) { $line[$x, $y] = $true; break }
            }
        }
    }
    for ($y = 0; $y -lt 32; $y++) {
        for ($x = 0; $x -lt 32; $x++) {
            if ($line[$x, $y]) { $bmp.SetPixel($ox + $x, $oy + $y, [System.Drawing.Color]::White) }
            elseif ((IsSet $line ($x - 1) ($y - 1))) { $bmp.SetPixel($ox + $x, $oy + $y, [System.Drawing.Color]::FromArgb(255, 120, 120, 120)) }
        }
    }
}

# A pistol facing right whose barrel starts at x: slide, front sight, slanted grip, trigger guard.
function Pistol($x) {
    return @{
        add = @(@('r', $x, 10, ($x + 16), 14), @('r', ($x + 15), 9, ($x + 15), 9), @('r', ($x + 1), 15, ($x + 5), 22),
            @('r', $x, 20, ($x + 4), 25), @('r', ($x + 6), 15, ($x + 9), 18))
        cut = @(, @('r', ($x + 7), 15, ($x + 8), 17))
    }
}
$gunRight = Pistol 13
$gunLeft = Pistol 2
$shoot = Pistol 3
$bold = @{
    '0,0' = @{ add = @(, @('t', 8, 8, 23, 8, 8, 23)) }
    '1,0' = @{ add = @(, @('t', 16, 5, 27, 21, 5, 21)) }
    '2,0' = @{ add = @(, @('t', 24, 8, 24, 23, 9, 8)) }
    # Spark lines at the muzzle rather than a triangle, so it doesn't look like the next-weapon icon.
    '3,0' = @{ add = $shoot.add + @(@('r', 23, 11, 29, 12), @('r', 23, 7, 24, 8), @('r', 25, 5, 26, 6), @('r', 23, 15, 24, 16), @('r', 25, 17, 26, 18)); cut = $shoot.cut }
    '0,1' = @{ add = @(, @('t', 5, 16, 21, 5, 21, 27)) }
    '2,1' = @{ add = @(, @('t', 27, 16, 11, 5, 11, 27)) }
    '3,1' = @{ add = @(@('t', 16, 3, 28, 15, 4, 15), @('r', 12, 15, 19, 27)) }
    '0,2' = @{ add = @(, @('t', 8, 24, 8, 9, 23, 24)) }
    '1,2' = @{ add = @(, @('t', 16, 27, 5, 11, 27, 11)) }
    '2,2' = @{ add = @(, @('t', 24, 24, 9, 24, 24, 9)) }
    '3,2' = @{ add = @(@('r', 6, 6, 26, 19), @('t', 9, 19, 16, 19, 8, 26)); cut = @(@('r', 15, 8, 17, 13), @('r', 15, 15, 17, 17)) }
    '0,3' = @{ add = @(@('r', 7, 7, 14, 14), @('r', 17, 7, 24, 14), @('r', 7, 17, 14, 24), @('r', 17, 17, 24, 24)) }
    '1,3' = @{ add = @(@('r', 9, 7, 13, 24), @('r', 18, 7, 22, 24)) }
    '2,3' = @{ add = @(@('t', 5, 7, 15, 15.5, 5, 24), @('t', 16, 7, 26, 15.5, 16, 24)) }
}
$prev = @{ add = @(, @('t', 2, 16, 10, 8, 10, 24)) + $gunRight.add; cut = $gunRight.cut }
$next = @{ add = @(, @('t', 30, 16, 22, 8, 22, 24)) + $gunLeft.add; cut = $gunLeft.cut }
# A map folded in three panels, the middle one lower, with a pin in it.
$map = @{ add = @(@('r', 5, 7, 11, 22), @('r', 13, 9, 18, 24), @('r', 20, 7, 26, 22)); cut = @(@('r', 14, 12, 17, 15), @('t', 14, 15, 18, 15, 16, 19)) }
# A padlock for the strafe lock: shackle with rounded top corners, body, keyhole.
$lock = @{
    add = @(@('r', 12, 3, 19, 5), @('r', 10, 5, 12, 13), @('r', 19, 5, 21, 13), @('r', 11, 4, 11, 4), @('r', 20, 4, 20, 4),
        @('r', 7, 13, 24, 27))
    cut = @(, @('r', 15, 18, 16, 22))
}

$atlas = New-Object System.Drawing.Bitmap 128, 320, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)

# Rows 0-3: upstream's icons, recoloured to white by brightness so tinting gives the chosen colour.
$classic = [System.Drawing.Bitmap]::FromFile((Join-Path $Dir "touch.png"))
for ($y = 0; $y -lt 128; $y++) {
    for ($x = 0; $x -lt 128; $x++) {
        $c = $classic.GetPixel($x, $y)
        if ($c.A -eq 0) { continue }
        $lum = [int][Math]::Min(255, (0.299 * $c.R + 0.587 * $c.G + 0.114 * $c.B) * 255.0 / 175.0)
        $atlas.SetPixel($x, $y, [System.Drawing.Color]::FromArgb($c.A, $lum, $lum, $lum))
    }
}
$classic.Dispose()

foreach ($key in $bold.Keys) {
    $cx, $cy = $key.Split(',') | ForEach-Object { [int]$_ }
    if (-not $bold[$key].cut) { $bold[$key].cut = @() }
    DrawBold $atlas ($cx * 32) (128 + $cy * 32) (Mask $bold[$key])
}
foreach ($i in 0, 1) {
    $icon = @($prev, $next)[$i]
    $m = Mask $icon
    DrawClassic $atlas ($i * 32) 256 $m
    DrawBold $atlas (64 + $i * 32) 256 $m
}
$m = Mask $map
DrawClassic $atlas 0 288 $m
DrawBold $atlas 32 288 $m
$m = Mask $lock
DrawClassic $atlas 64 288 $m
DrawBold $atlas 96 288 $m
$atlas.Save((Join-Path $Dir "touch_icons.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$atlas.Dispose()

# Backings: dark see-through fill with a white rim, plus solid versions for the pressed glow.
$plate = New-Object System.Drawing.Bitmap 96, 96, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$fill = [System.Drawing.Color]::FromArgb(150, 12, 14, 24)
$rim = [System.Drawing.Color]::FromArgb(210, 255, 255, 255)
$solid = [System.Drawing.Color]::White
for ($y = 0; $y -lt 48; $y++) {
    for ($x = 0; $x -lt 48; $x++) {
        # Round: distance from the centre.
        $d = [Math]::Sqrt(($x - 23.5) * ($x - 23.5) + ($y - 23.5) * ($y - 23.5))
        $inside = 23.5 - $d
        # Rounded square: distance inside the edge, with 10px corners.
        $qx = [Math]::Max([Math]::Abs($x - 23.5) - 13.5, 0); $qy = [Math]::Max([Math]::Abs($y - 23.5) - 13.5, 0)
        $sq = 10.0 - [Math]::Sqrt($qx * $qx + $qy * $qy)
        if ($qx -eq 0 -and $qy -eq 0) { $sq = 10.0 + [Math]::Min(13.5 - [Math]::Abs($x - 23.5), 13.5 - [Math]::Abs($y - 23.5)) }

        if ($inside -ge 0) {
            $plate.SetPixel($x, $y, $(if ($inside -lt 2.0) { $rim } else { $fill }))
            $plate.SetPixel(48 + $x, $y, $solid)
        }
        if ($sq -ge 0) {
            $plate.SetPixel($x, 48 + $y, $(if ($sq -lt 2.0) { $rim } else { $fill }))
            $plate.SetPixel(48 + $x, 48 + $y, $solid)
        }
    }
}
$plate.Save((Join-Path $Dir "touch_plate.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$plate.Dispose()
Write-Output "wrote touch_icons.png and touch_plate.png in $Dir"
