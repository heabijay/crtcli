using System.Collections.Specialized;
using System.Reflection;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class GlobalAppSettingsExtensions
{
    private static readonly Type GlobalAppSettingsType = typeof(GlobalAppSettings);
    
    private static readonly MethodInfo InitializeMethod =
        GlobalAppSettingsType.GetMethod(
            "Initialize",
            BindingFlags.Static | BindingFlags.NonPublic, null, [typeof(NameValueCollection)], null)
        ?? throw new MissingMethodException(GlobalAppSettingsType.FullName, "Init");
    
    public static void Initialize(NameValueCollection config)
    {
        InitializeMethod.Invoke(null, [config]);
    }
}