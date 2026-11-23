# YALNIZ SINAMA: Tauri deneme uygulamasında WebView2'nin ekran seçicisini UI Automation ile geçer.
# Yalnız 'Nahrok' başlıklı, verilen işleme ait pencereye bakar. Her seçicide: "Tüm Ekran" sekmesi, ilk ekran,
# varsa ses onay kutusu işaretlenir, "Paylaş". Ne gördüğünü çıktı dosyasına yazar (UTF-8).
param([string]$Islem = 'nahrok-desktop-deneme', [int]$Adet = 2, [string]$Cikti = "$env:TEMP\webview2-secici.txt")
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A = [Windows.Automation.AutomationElement]; $T = [Windows.Automation.TreeScope]; $CT = [Windows.Automation.ControlType]
$log = New-Object System.Collections.Generic.List[string]
function Kosul($tip) { New-Object Windows.Automation.PropertyCondition($A::ControlTypeProperty, $tip) }

for ($n = 1; $n -le $Adet; $n++) {
  $secici = $null
  for ($i = 0; $i -lt 60 -and -not $secici; $i++) {
    Start-Sleep -Milliseconds 500
    $pid0 = (Get-Process -Name $Islem -ErrorAction SilentlyContinue | Select-Object -First 1).Id
    if (-not $pid0) { continue }
    $ana = $A::RootElement.FindFirst($T::Children, (New-Object Windows.Automation.AndCondition(
      (New-Object Windows.Automation.PropertyCondition($A::ProcessIdProperty, $pid0)),
      (New-Object Windows.Automation.PropertyCondition($A::NameProperty, 'Nahrok')))))
    if ($ana) { $secici = $ana.FindFirst($T::Descendants, (Kosul $CT::Window)) }
  }
  if (-not $secici) { $log.Add("[$n] seçici bulunamadı"); break }
  $log.Add("[$n] seçici: $($secici.Current.Name)")

  $sekmeler = $secici.FindAll($T::Descendants, (Kosul $CT::TabItem))
  $log.Add("  sekmeler: " + (($sekmeler | ForEach-Object { $_.Current.Name }) -join ' | '))
  $son = $sekmeler[$sekmeler.Count - 1]
  $son.GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Select()
  Start-Sleep -Milliseconds 1200

  $ogeler = $secici.FindAll($T::Descendants, (Kosul $CT::ListItem))
  $log.Add("  kaynaklar: " + (($ogeler | ForEach-Object { $_.Current.Name }) -join ' | '))
  if ($ogeler.Count) {
    try { $ogeler[0].GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Select() } catch {
      try { $ogeler[0].GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { $log.Add('  kaynak seçilemedi') } }
  }
  Start-Sleep -Milliseconds 500
  $dugmeler = $secici.FindAll($T::Descendants, (Kosul $CT::Button))
  $log.Add("  düğmeler: " + (($dugmeler | ForEach-Object { "$($_.Current.Name)(etkin=$($_.Current.IsEnabled))" }) -join ' | '))
  # 1) kaynak kutucuğu (ilk düğme: "Tüm ekran"), 2) "Sistem sesiyle paylaş" açık olsun, 3) "Paylaş"
  $kaynak = $dugmeler | Select-Object -First 1
  $kaynak.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
  $log.Add("  kaynak: $($kaynak.Current.Name)")
  Start-Sleep -Milliseconds 600
  $ses = $secici.FindAll($T::Descendants, (Kosul $CT::Button)) | Where-Object { $_.Current.Name -match 'Sistem ses' } | Select-Object -First 1
  if ($ses) {
    $durum = 'bilinmiyor'
    try { $tg = $ses.GetCurrentPattern([Windows.Automation.TogglePattern]::Pattern); $durum = $tg.Current.ToggleState
          if ($durum -ne 'On') { $tg.Toggle() }; $log.Add("  sistem sesi: $durum -> $($tg.Current.ToggleState)") }
    catch { $ses.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke(); $log.Add('  sistem sesi düğmesine basıldı (toggle yok)') }
  } else { $log.Add('  sistem sesi seçeneği yok') }
  Start-Sleep -Milliseconds 600
  $paylas = $secici.FindAll($T::Descendants, (Kosul $CT::Button)) | Where-Object { $_.Current.Name -match 'payla' -and $_.Current.Name -notmatch 'Sistem' } | Select-Object -First 1
  if ($paylas -and $paylas.Current.IsEnabled) {
    $paylas.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke(); $log.Add("  basıldı: $($paylas.Current.Name)")
  } else {
    $log.Add('  Paylaş etkin değil; durum: ' + (($secici.FindAll($T::Descendants, (Kosul $CT::Button)) | ForEach-Object { "$($_.Current.Name)(etkin=$($_.Current.IsEnabled))" }) -join ' | '))
  }
  Start-Sleep -Seconds 3
}
[IO.File]::WriteAllLines($Cikti, $log, (New-Object Text.UTF8Encoding($false)))
