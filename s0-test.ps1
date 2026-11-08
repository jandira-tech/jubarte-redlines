$ErrorActionPreference = "Stop"
$out = "C:\word-endpoint\work\s0-result-$env:USERNAME.txt"
$work = "C:\word-endpoint\work\s0"
New-Item -ItemType Directory -Force -Path $work | Out-Null

$session = (Get-Process -Id $PID).SessionId
"user=$env:USERNAME session=$session" | Set-Content -Path $out

try {
    $word = New-Object -ComObject Word.Application
    $word.Visible = $false
    $word.DisplayAlerts = 0
    $doc = $word.Documents.Add()
    $null = $word.Selection.TypeText("session0 smoke test")
    $docx = Join-Path $work "probe.docx"
    $doc.SaveAs2($docx)
    $pdf = Join-Path $work "probe.pdf"
    $doc.ExportAsFixedFormat($pdf, 17)
    $doc.Close(0)
    $word.Quit()
    Start-Sleep -Seconds 3
    $pdfOk = (Test-Path $pdf) -and ((Get-Item $pdf).Length -gt 1000)
    if ($pdfOk) { "OK docx+pdf written" | Add-Content -Path $out }
    else { "FAIL pdf missing or tiny" | Add-Content -Path $out }
} catch {
    "FAIL: $($_.Exception.Message)" | Add-Content -Path $out
    try { Get-Process WINWORD -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $session } | Stop-Process -Force } catch {}
}
"done" | Add-Content -Path $out
