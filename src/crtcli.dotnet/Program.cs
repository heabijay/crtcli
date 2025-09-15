using System.Reflection;
using System.Runtime.Loader;
using System.Text.Json;
using System.Text.Json.Serialization;
using CrtCli.Dotnet;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

static void ConfigureLogging(ILoggingBuilder configure)
{
    configure.AddSimpleConsole(options =>
    {
        options.SingleLine = true;
    });
}

using var loggerFactory = LoggerFactory.Create(ConfigureLogging);

var logger = loggerFactory.CreateLogger<Program>();

if (!EnsureVersionFileHasCorrectData() && !string.IsNullOrEmpty(Environment.GetEnvironmentVariable(Constants.EnvironmentVariables.ThrowExceptionOnVersionFileMismatch)))
{
    logger.LogError(
        "Application is exiting due to version file was updated and {EnvironmentVariable} environment variable is present. Please run this application again, just in case you need to know correct version!", 
        Constants.EnvironmentVariables.ThrowExceptionOnVersionFileMismatch);

    return (int)Constants.ExitCode.VersionFileHasBeenChanged;
}

if (string.IsNullOrEmpty(Environment.GetEnvironmentVariable(Constants.EnvironmentVariables.CalledByCrtCli)))
{
    logger.LogWarning("You're calling CrtCli.Dotnet directly. This utility is designed to be used with 'crtcli'. Please use it instead!");
}

if (args.Length < 2)
{
    logger.LogInformation("Usage: crtcli-dotnet <command> <payload>");
    return (int)Constants.ExitCode.ArgumentParsingFailed;
}

var command = args[0];
var payload = args[1];


var serviceCollection = new ServiceCollection();

serviceCollection.AddLogging(ConfigureLogging);


// TODO: handle cannot resolve dll, creatio folder from env/req, package folder etc
const string creatioSdkDir = "/home/User/Creatio/8.3.0.3070_SEMS_Net8_PostgreSQL";
const string packageDir = "/home/User/source/repos/creatio_package";

if (File.Exists(Path.Combine(creatioSdkDir, "Global.asax")))
{
    Console.WriteLine(".NET Framework (IIS) based Creatio is not supported! Please use .NET Core / .NET Kestrel based Creatio instance.");
}

AssemblyLoadContext.Default.Resolving += (ctx, name) =>
{
    var targetFile = Path.Combine(creatioSdkDir, name.Name + ".dll");
        
    return File.Exists(targetFile) ? ctx.LoadFromAssemblyPath(targetFile) : null;
};

// var assembly = assemblyLoadContext.LoadFromAssemblyName(new AssemblyName("Terrasoft.Core"));
// var userConnection = assembly.GetType("Terrasoft.Core.UserConnection");

CrtCli.Dotnet.InternalV2.InternalMain(packageDir);

return (int)Constants.ExitCode.Ok;


bool EnsureVersionFileHasCorrectData()
{
    var executingAssembly = Assembly.GetExecutingAssembly();
    var binaryFolder = Path.GetDirectoryName(executingAssembly.Location)!;
    var versionFilepath = Path.Combine(binaryFolder, Constants.VersionFilename);

    var storedVersion = ReadVersionFile();
    var actualVersion = executingAssembly.GetName().Version;

    if (storedVersion?.Version != actualVersion)
    {
        logger.LogWarning(
            "Stored version in file {VersionFilename} is mismatched: stored {StoredVersion} <> actual {ActualVersion}. Stored version will be updated", 
            Constants.VersionFilename,
            storedVersion, 
            actualVersion);
        
        WriteVersionFile();

        return false;
    }
    
    return true;


    VersionFileModel? ReadVersionFile()
    {
        if (!File.Exists(versionFilepath))
        {
            return null;
        }
        
        var json = File.ReadAllText(versionFilepath);

        if (string.IsNullOrEmpty(json))
        {
            return null;
        }
        
        try
        {
            return JsonSerializer.Deserialize<VersionFileModel>(json);
        }
        catch (JsonException ex)
        {
            logger.LogWarning(
                ex, 
                "Cannot parse the file {VersionFilename} located at {VersionFilepath}. Stored version will be assumed to be undefined", 
                Constants.VersionFilename, 
                versionFilepath);

            return null;
        }
    }
    
    void WriteVersionFile()
    {
        File.WriteAllText(versionFilepath, JsonSerializer.Serialize(new VersionFileModel(actualVersion!)));

        logger.LogDebug("Version in file {VersionFilepath} was updated to actual value", versionFilepath);
    }
}

record VersionFileModel([property: JsonPropertyName("version")] Version Version);