// Task 22
use crate::error::JError;
use crate::install::region;
use std::path::Path;

pub fn build_shim_script(exe_abs: &str) -> String {
    let exe_q = exe_abs.replace('\'', "''");
    format!(
        r#"function j {{
    $script:_jExe = '{exe}'
    # j.exe always emits UTF-8. Windows PowerShell 5.1 reads a child process's
    # stdout using the OEM console code page, which mangles non-ASCII (e.g. CJK)
    # paths — making `Set-Location` fail and listings unreadable. Force UTF-8 for
    # the duration of this call (covers the jump output, :list display, and the
    # interactive picker), then restore the previous encoding in finally.
    $script:_jPrevEnc = $null
    try {{ $script:_jPrevEnc = [Console]::OutputEncoding; [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 }} catch {{}}
    try {{
    function _jRun($toks) {{
        if (-not $toks -or $toks.Count -eq 0) {{ return }}
        $o = (& $script:_jExe --shell=powershell @toks) -join "`n"
        if ($LASTEXITCODE -eq 0 -and $o -match '^Set-Location') {{ Invoke-Expression $o }}
        elseif ($LASTEXITCODE -eq 0 -and $o) {{ Write-Host $o }}
    }}
    if ($args.Count -gt 0) {{ _jRun $args; return }}

    function _jParseArgv($s) {{
        if ([string]::IsNullOrWhiteSpace($s)) {{ return ,@() }}
        $tokens = $null; $errs = $null
        [void][System.Management.Automation.Language.Parser]::ParseInput("j $s", [ref]$tokens, [ref]$errs)
        $out = [System.Collections.ArrayList]::new()
        $skipFirst = $true
        foreach ($t in $tokens) {{
            $k = $t.Kind.ToString()
            if ($k -in 'EndOfInput','NewLine','LineContinuation','Comment','Semi') {{ continue }}
            if ($skipFirst) {{ $skipFirst = $false; continue }}
            if ($t -is [System.Management.Automation.Language.StringToken]) {{ [void]$out.Add($t.Value) }}
            else {{ [void]$out.Add($t.Text) }}
        }}
        return ,$out.ToArray()
    }}
    function _jWidth($s) {{
        if (-not $s) {{ return 0 }}
        $w = 0
        foreach ($c in $s.ToCharArray()) {{
            $cp = [int]$c
            if (($cp -ge 0x1100 -and $cp -le 0x115F) -or
                ($cp -ge 0x2E80 -and $cp -le 0x303E) -or
                ($cp -ge 0x3041 -and $cp -le 0x33FF) -or
                ($cp -ge 0x3400 -and $cp -le 0x4DBF) -or
                ($cp -ge 0x4E00 -and $cp -le 0x9FFF) -or
                ($cp -ge 0xA000 -and $cp -le 0xA4CF) -or
                ($cp -ge 0xAC00 -and $cp -le 0xD7A3) -or
                ($cp -ge 0xF900 -and $cp -le 0xFAFF) -or
                ($cp -ge 0xFE30 -and $cp -le 0xFE4F) -or
                ($cp -ge 0xFF00 -and $cp -le 0xFF60) -or
                ($cp -ge 0xFFE0 -and $cp -le 0xFFE6)) {{ $w += 2 }}
            else {{ $w += 1 }}
        }}
        return $w
    }}
    function _jCursorCol {{
        $c = 2 + (_jWidth $script:_jBuf.Substring(0, $script:_jPos))
        $maxC = [Console]::WindowWidth - 1
        if ($c -gt $maxC) {{ $c = $maxC }}
        return $c
    }}

    $script:_jBuf = ''; $script:_jPos = 0; $script:_jSelIdx = -1; $script:_jNDrawn = 0
    # Reserve room below the prompt so candidates never render off-screen, even near window bottom.
    $needed = 11
    $winBottom = [Console]::WindowTop + [Console]::WindowHeight - 1
    if ([Console]::CursorTop + $needed -gt $winBottom) {{
        $scroll = [Console]::CursorTop + $needed - $winBottom
        for ($i = 0; $i -lt $scroll; $i++) {{ [Console]::WriteLine() }}
        [Console]::SetCursorPosition(0, [Console]::CursorTop - $scroll)
    }}
    $script:_jTop = [Console]::CursorTop

    function _jFetch {{
        $ln = "j $($script:_jBuf)"; $cur = $ln.Length
        $raw = & $script:_jExe :complete-rich powershell $cur $ln 2>$null
        $r = [System.Collections.Generic.List[PSCustomObject]]::new()
        foreach ($x in $raw) {{
            if (-not $x) {{ continue }}
            $p = $x.Split("`t", 2)
            if ($p[0]) {{ $r.Add([PSCustomObject]@{{ Sym = $p[0]; Path = if ($p.Count -ge 2) {{ $p[1] }} else {{ '' }} }}) }}
        }}
        return ,$r.ToArray()
    }}
    function _jClearBelow {{
        if ($script:_jNDrawn -eq 0) {{ return }}
        $st = [Console]::CursorTop; $sl = [Console]::CursorLeft; $w = [Console]::WindowWidth
        $winBot = [Console]::WindowTop + [Console]::WindowHeight - 1
        for ($i = 1; $i -le $script:_jNDrawn; $i++) {{
            $row = $script:_jTop + $i
            if ($row -gt $winBot) {{ break }}
            [Console]::SetCursorPosition(0, $row)
            [Console]::Write(' ' * $w)
        }}
        [Console]::SetCursorPosition($sl, $st); $script:_jNDrawn = 0
    }}
    function _jDraw($cands, $sel) {{
        _jClearBelow
        if (-not $cands -or $cands.Count -eq 0) {{ return }}
        $w = [Console]::WindowWidth
        $winBot = [Console]::WindowTop + [Console]::WindowHeight - 1
        $avail = [Math]::Max(0, $winBot - $script:_jTop - 1)
        $show = [Math]::Min([Math]::Min($cands.Count, 10), $avail)
        if ($show -le 0) {{ return }}
        $extra = if ($cands.Count -gt $show -and ($show + 1) -le $avail) {{ 1 }} else {{ 0 }}
        $script:_jNDrawn = $show + $extra
        $st = [Console]::CursorTop; $sl = [Console]::CursorLeft
        $oldFg = [Console]::ForegroundColor
        for ($i = 0; $i -lt $show; $i++) {{
            $sym = $cands[$i].Sym; $path = $cands[$i].Path
            [Console]::SetCursorPosition(0, $script:_jTop + 1 + $i)
            $sp = "  $sym"
            if ($path) {{
                $ar = ' -> '; $av = $w - $sp.Length - $ar.Length - 1
                if ($av -gt 3 -and $path.Length -gt $av) {{
                    $path = '...' + $path.Substring($path.Length - ($av - 3))
                }} elseif ($av -le 3) {{ $path = '' }}
                $text = if ($path) {{ "$sp$ar$path" }} else {{ $sp }}
            }} else {{ $text = $sp }}
            $text = $text.PadRight($w).Substring(0, $w)
            if ($i -eq $sel) {{ [Console]::ForegroundColor = [ConsoleColor]::Cyan }}
            [Console]::Write($text)
            if ($i -eq $sel) {{ [Console]::ForegroundColor = $oldFg }}
        }}
        if ($extra) {{
            [Console]::SetCursorPosition(0, $script:_jTop + 1 + $show)
            $rem = $cands.Count - $show
            [Console]::Write("  ... ($rem more)".PadRight($w).Substring(0, $w))
        }}
        [Console]::SetCursorPosition($sl, $st)
    }}
    function _jRedraw {{
        [Console]::SetCursorPosition(0, $script:_jTop)
        $ln = 'j ' + $script:_jBuf; $w = [Console]::WindowWidth
        [Console]::Write($ln.PadRight($w).Substring(0, $w))
        [Console]::SetCursorPosition((_jCursorCol), $script:_jTop)
    }}
    [Console]::Write('j ')
    $cands = _jFetch; _jDraw $cands $script:_jSelIdx
    while ($true) {{
        $k = [Console]::ReadKey($true); $key = $k.Key; $ch = $k.KeyChar; $mod = $k.Modifiers
        if ($key -eq [ConsoleKey]::Enter) {{
            _jClearBelow; [Console]::WriteLine()
            $toks = _jParseArgv $script:_jBuf
            if ($toks.Count -gt 0) {{ _jRun $toks }}
            return
        }}
        if ($key -eq [ConsoleKey]::C -and ($mod -band [ConsoleModifiers]::Control)) {{
            _jClearBelow; [Console]::WriteLine(); return
        }}
        if ($key -eq [ConsoleKey]::Escape) {{
            $script:_jSelIdx = -1; _jDraw $cands $script:_jSelIdx; continue
        }}
        if ($key -eq [ConsoleKey]::Tab) {{
            if ($cands -and $cands.Count -gt 0) {{
                $s = $script:_jSelIdx
                $chosen = if ($s -ge 0 -and $s -lt $cands.Count) {{ $cands[$s].Sym }} else {{ $cands[0].Sym }}
                if ($script:_jBuf -match '^(.*\s)\S*$') {{ $script:_jBuf = $Matches[1] + $chosen + ' ' }}
                else {{ $script:_jBuf = $chosen + ' ' }}
                $script:_jPos = $script:_jBuf.Length; $script:_jSelIdx = -1
                $cands = _jFetch; _jRedraw; _jDraw $cands $script:_jSelIdx
            }}
            continue
        }}
        if ($key -eq [ConsoleKey]::UpArrow) {{
            if ($cands -and $cands.Count -gt 0) {{
                $script:_jSelIdx = if ($script:_jSelIdx -gt 0) {{ $script:_jSelIdx - 1 }} elseif ($script:_jSelIdx -eq 0) {{ -1 }} else {{ $cands.Count - 1 }}
                _jDraw $cands $script:_jSelIdx
            }}
            continue
        }}
        if ($key -eq [ConsoleKey]::DownArrow) {{
            if ($cands -and $cands.Count -gt 0) {{
                $script:_jSelIdx = if ($script:_jSelIdx -eq $cands.Count - 1) {{ -1 }}
                    elseif ($script:_jSelIdx -eq -1) {{ 0 }} else {{ $script:_jSelIdx + 1 }}
                _jDraw $cands $script:_jSelIdx
            }}
            continue
        }}
        if ($key -eq [ConsoleKey]::LeftArrow) {{
            if ($script:_jPos -gt 0) {{ $script:_jPos--; [Console]::SetCursorPosition((_jCursorCol), $script:_jTop) }}
            continue
        }}
        if ($key -eq [ConsoleKey]::RightArrow) {{
            if ($script:_jPos -lt $script:_jBuf.Length) {{ $script:_jPos++; [Console]::SetCursorPosition((_jCursorCol), $script:_jTop) }}
            continue
        }}
        if ($key -eq [ConsoleKey]::Backspace) {{
            if ($script:_jPos -gt 0) {{
                $script:_jBuf = $script:_jBuf.Substring(0, $script:_jPos - 1) + $script:_jBuf.Substring($script:_jPos)
                $script:_jPos--; $script:_jSelIdx = -1
                $cands = _jFetch; _jRedraw; _jDraw $cands $script:_jSelIdx
            }}
            continue
        }}
        if ($ch -and $ch -ne [char]0 -and [int]$ch -ge 32) {{
            $script:_jBuf = $script:_jBuf.Substring(0, $script:_jPos) + $ch + $script:_jBuf.Substring($script:_jPos)
            $script:_jPos++
            # Paste drain: absorb pending printable keys so _jFetch fires once per burst.
            while ([Console]::KeyAvailable) {{
                $k2 = [Console]::ReadKey($true); $c2 = $k2.KeyChar
                if (-not $c2 -or [int]$c2 -lt 32) {{ break }}
                if ($k2.Key -eq [ConsoleKey]::Enter -or $k2.Key -eq [ConsoleKey]::Tab) {{ break }}
                $script:_jBuf = $script:_jBuf.Substring(0, $script:_jPos) + $c2 + $script:_jBuf.Substring($script:_jPos)
                $script:_jPos++
            }}
            $script:_jSelIdx = -1
            $cands = _jFetch; _jRedraw; _jDraw $cands $script:_jSelIdx
        }}
    }}
    }} finally {{
        if ($script:_jPrevEnc) {{ try {{ [Console]::OutputEncoding = $script:_jPrevEnc }} catch {{}} }}
    }}
}}
Register-ArgumentCompleter -CommandName j -ScriptBlock {{
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    if ($commandAst) {{
        $ln = $commandAst.ToString(); $cur = $ln.Length
    }} else {{
        $ln = $parameterName; $cur = [int]$wordToComplete
    }}
    $_jPrevEnc = $null
    try {{ $_jPrevEnc = [Console]::OutputEncoding; [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 }} catch {{}}
    try {{
        $out = @(& '{exe}' :complete powershell $cur $ln 2>$null)
    }} finally {{
        if ($_jPrevEnc) {{ try {{ [Console]::OutputEncoding = $_jPrevEnc }} catch {{}} }}
    }}
    foreach ($x in $out) {{
        if ($x) {{ [System.Management.Automation.CompletionResult]::new($x, $x, 'ParameterValue', $x) }}
    }}
}}
"#,
        exe = exe_q
    )
}

pub fn install_into_file(profile: &Path, exe_abs: &str) -> Result<(), JError> {
    let existing = std::fs::read_to_string(profile).unwrap_or_default();
    if let Some(parent) = profile.parent() {
        std::fs::create_dir_all(parent).map_err(|e| JError::InstallError {
            msg: format!("mkdir {}: {}", parent.display(), e),
        })?;
    }
    let body = build_shim_script(exe_abs);
    let updated = region::upsert(&existing, &body);
    std::fs::write(profile, updated).map_err(|e| JError::InstallError {
        msg: format!("write {}: {}", profile.display(), e),
    })
}

pub fn uninstall_from_file(profile: &Path) -> Result<(), JError> {
    let existing = match std::fs::read_to_string(profile) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    let updated = region::remove(&existing)?;
    std::fs::write(profile, updated).map_err(|e| JError::InstallError {
        msg: format!("write {}: {}", profile.display(), e),
    })
}

pub fn default_profile_path() -> Result<std::path::PathBuf, JError> {
    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").map_err(|_| JError::InstallError {
            msg: "HOME not set".into(),
        })?;
        Ok(std::path::PathBuf::from(home)
            .join(".config")
            .join("powershell")
            .join("Microsoft.PowerShell_profile.ps1"))
    }

    #[cfg(windows)]
    {
        let home = std::env::var("USERPROFILE").map_err(|_| JError::InstallError {
            msg: "USERPROFILE not set".into(),
        })?;
        Ok(std::path::PathBuf::from(home)
            .join("Documents")
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"))
    }
}

/// Returns profile paths for both Windows PowerShell 5.1 and PowerShell 7.
/// Install targets all of them so the shim works regardless of PS version.
pub fn all_profile_paths() -> Result<Vec<std::path::PathBuf>, JError> {
    #[cfg(not(windows))]
    {
        Ok(vec![default_profile_path()?])
    }

    #[cfg(windows)]
    {
        let home = std::env::var("USERPROFILE").map_err(|_| JError::InstallError {
            msg: "USERPROFILE not set".into(),
        })?;
        let base = std::path::PathBuf::from(home).join("Documents");
        Ok(vec![
            base.join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
            base.join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        ])
    }
}
