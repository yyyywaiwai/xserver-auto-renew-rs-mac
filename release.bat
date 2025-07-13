@echo off
REM ------------------------------------------------------------
REM Cargo.toml の version を取得して
REM   直前のコミット (HEAD~1) に v{ver} タグを付けてプッシュする
REM ------------------------------------------------------------
setlocal EnableDelayedExpansion

REM ▼ 1. Cargo.toml から version を抽出 -------------------------
for /f "usebackq tokens=2 delims==" %%A in (
  `findstr /R /C:"^\s*version\s*=" Cargo.toml`
) do (
  set "rawver=%%A"
  goto :verfound
)

echo [ERROR] Cargo.toml で version を見つけられませんでした。
exit /b 1

:verfound
REM 空白を除去
set "rawver=!rawver: =!"
REM ダブルクオートを除去
set "ver=!rawver:"=!"

if "!ver!"=="" (
  echo [ERROR] version の読み取りに失敗しました。
  exit /b 1
)

echo [INFO] 取得したバージョン: !ver!

REM ▼ 2. 同名タグの重複チェック --------------------------------
git show-ref --tags --quiet --verify "refs/tags/v!ver!"
if not errorlevel 1 (
  echo [ERROR] タグ v!ver! は既に存在します。
  exit /b 1
)

REM ▼ 3. 直前のコミットにタグ付け --------------------------------
git tag -a "v!ver!" HEAD~1 -m "Release v!ver!"
if errorlevel 1 (
  echo [ERROR] タグ付けに失敗しました。
  exit /b 1
)

REM ▼ 4. リモートへプッシュ ------------------------------------
git push origin "v!ver!"
if errorlevel 1 (
  echo [ERROR] プッシュに失敗しました。
  exit /b 1
)

echo [SUCCESS] タグ v!ver! を HEAD~1 に付けてリモートへプッシュしました。
endlocal
