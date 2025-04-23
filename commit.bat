@echo off

if "%~1"=="" (
 echo.
 echo ------------------------
 echo Git commit ^& push script
 echo ------------------------
 echo.
 echo This script commits the changes and pushes them to GitHub.
 echo.
 echo Usage: %~nx0 "[SOME COMMENT]"
 echo Example: %~nx0 "Add README.md"
 echo.
 exit /b 1
)

git status
git add .
git status
git commit -m "%~1"
git push
git status
