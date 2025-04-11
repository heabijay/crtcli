rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
check_always_make=$(findstring B,$(MAKEFLAGS))

.PHONY: build-all clear-before-rebuild

build-all: clear-before-rebuild Files/Bin/crtcli.tunneling.dll Files/Bin/netstandard/crtcli.tunneling.dll

Files/Bin/netstandard/crtcli.tunneling.dll: $(call rwildcard,src,*) CrtCli.Packages.Tunneling.csproj
	dotnet build -c Release -p:CoreTargetFramework=netstandard2.0

Files/Bin/crtcli.tunneling.dll: $(call rwildcard,src,*) CrtCli.Packages.Tunneling.csproj
	dotnet build -c Release -p:CoreTargetFramework=net472

clear-before-rebuild:
	@if [ -n "$(check_always_make)" ]; then \
		rm -rf Files/Bin; \
	fi