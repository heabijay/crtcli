namespace CrtCli.Dotnet;

internal static class Constants
{
    public const string VersionFilename = "CrtCli.Dotnet.version.json";

    public enum ExitCode
    {
        Ok = 0,
        Error = 1,
        VersionFileHasBeenChanged = 2,
        ArgumentParsingFailed = 3,
    }
    
    public static class EnvironmentVariables
    {
        private const string Prefix = "CRTCLI_DOTNET_";

        public const string ThrowExceptionOnVersionFileMismatch = $"{Prefix}THROW_EXCEPTION_ON_VERSION_FILE_MISMATCH";
        
        public const string CalledByCrtCli = $"{Prefix}CALLED_BY_CRTCLI";
    }
}