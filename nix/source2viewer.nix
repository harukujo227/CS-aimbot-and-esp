{ fetchFromGitHub
, buildDotnetModule
, dotnetCorePackages
, ... }:
buildDotnetModule rec {
    pname = "ValveResourceFormat";
    version = "15.0";

    src = fetchFromGitHub {
      owner = pname;
      repo = pname;
      rev = "aa1c384b9481a6892b3963d96251b3065191db47";
      sha256 = "sha256-1p7AeiZ5Bgre77XuS5xjwjk4Kzs0spEh2e/WHLuQG5I=";
    };

    nugetDeps = ./deps.json;

  dotnet-sdk = dotnetCorePackages.sdk_9_0;
  dotnet-runtime = dotnetCorePackages.runtime_9_0;

  projectFile = "./CLI/CLI.csproj";
}