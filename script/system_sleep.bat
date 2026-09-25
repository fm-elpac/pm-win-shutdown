:: 先执行 自定义任务
cmd.exe /C sleep_before.bat

:: 睡眠
rundll32.exe powrprof.dll,SetSuspendState Sleep

:: 安装位置 %LOCALAPPDATA%\pm-win-shutdown\script\
:: 快捷方式 %APPDATA%\Microsoft\Windows\Start Menu\Programs\pm-win-shutdown\
