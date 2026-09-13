Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$samplePath = Join-Path $root "examples\hello.apl"
$tempPath = Join-Path $root ".apl_gui_run.apl"

$form = New-Object System.Windows.Forms.Form
$form.Text = "APL Runner"
$form.Size = New-Object System.Drawing.Size(1100, 760)
$form.StartPosition = "CenterScreen"

$codeLabel = New-Object System.Windows.Forms.Label
$codeLabel.Text = "Code"
$codeLabel.Location = New-Object System.Drawing.Point(10, 10)
$codeLabel.Size = New-Object System.Drawing.Size(500, 20)
$form.Controls.Add($codeLabel)

$codeBox = New-Object System.Windows.Forms.TextBox
$codeBox.Multiline = $true
$codeBox.ScrollBars = "Both"
$codeBox.AcceptsTab = $true
$codeBox.WordWrap = $false
$codeBox.Font = New-Object System.Drawing.Font("Consolas", 10)
$codeBox.Location = New-Object System.Drawing.Point(10, 35)
$codeBox.Size = New-Object System.Drawing.Size(660, 650)
if (Test-Path $samplePath) {
    $codeBox.Text = Get-Content $samplePath -Raw
}
$form.Controls.Add($codeBox)

$inputLabel = New-Object System.Windows.Forms.Label
$inputLabel.Text = "Input lines"
$inputLabel.Location = New-Object System.Drawing.Point(685, 10)
$inputLabel.Size = New-Object System.Drawing.Size(390, 20)
$form.Controls.Add($inputLabel)

$inputBox = New-Object System.Windows.Forms.TextBox
$inputBox.Multiline = $true
$inputBox.ScrollBars = "Vertical"
$inputBox.Font = New-Object System.Drawing.Font("Consolas", 10)
$inputBox.Location = New-Object System.Drawing.Point(685, 35)
$inputBox.Size = New-Object System.Drawing.Size(380, 140)
$form.Controls.Add($inputBox)

$runButton = New-Object System.Windows.Forms.Button
$runButton.Text = "Run"
$runButton.Location = New-Object System.Drawing.Point(685, 190)
$runButton.Size = New-Object System.Drawing.Size(90, 34)
$form.Controls.Add($runButton)

$checkButton = New-Object System.Windows.Forms.Button
$checkButton.Text = "Check"
$checkButton.Location = New-Object System.Drawing.Point(785, 190)
$checkButton.Size = New-Object System.Drawing.Size(90, 34)
$form.Controls.Add($checkButton)

$outputLabel = New-Object System.Windows.Forms.Label
$outputLabel.Text = "Console"
$outputLabel.Location = New-Object System.Drawing.Point(685, 240)
$outputLabel.Size = New-Object System.Drawing.Size(390, 20)
$form.Controls.Add($outputLabel)

$outputBox = New-Object System.Windows.Forms.TextBox
$outputBox.Multiline = $true
$outputBox.ScrollBars = "Both"
$outputBox.ReadOnly = $true
$outputBox.WordWrap = $false
$outputBox.Font = New-Object System.Drawing.Font("Consolas", 10)
$outputBox.Location = New-Object System.Drawing.Point(685, 265)
$outputBox.Size = New-Object System.Drawing.Size(380, 420)
$form.Controls.Add($outputBox)

function Invoke-AplCommand {
    param([string]$CommandName)

    Set-Content -Path $tempPath -Value $codeBox.Text -Encoding UTF8
    $outputBox.Text = "Running apl $CommandName..."

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "cargo"
    $psi.Arguments = "run -p apl -- $CommandName `"$tempPath`""
    $psi.WorkingDirectory = $root
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.RedirectStandardInput = $true
    $psi.CreateNoWindow = $true

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $psi
    [void]$process.Start()

    if ($CommandName -eq "run") {
        $process.StandardInput.Write($inputBox.Text)
    }
    $process.StandardInput.Close()

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    $outputBox.Text = "Exit code: $($process.ExitCode)`r`n`r`nSTDOUT:`r`n$stdout`r`nSTDERR:`r`n$stderr"
}

$runButton.Add_Click({ Invoke-AplCommand "run" })
$checkButton.Add_Click({ Invoke-AplCommand "check" })

[void]$form.ShowDialog()
