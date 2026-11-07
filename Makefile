.DEFAULT_GOAL := windows

# Arguments supplémentaires du script PowerShell, par exemple -CompilerBin.
WINDOWS_ARGS ?=

ifeq ($(OS),Windows_NT)
WINDOWS_BUILD = powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows.ps1
else
WINDOWS_BUILD = bash scripts/build-windows.sh
endif

.PHONY: all windows windows-debug

all: windows

windows:
	$(WINDOWS_BUILD) $(WINDOWS_ARGS)

windows-debug:
	$(WINDOWS_BUILD) -DebugBuild $(WINDOWS_ARGS)
