using System.Collections.Specialized;
using CrtCli.Dotnet.Mocking.Extensions;

namespace CrtCli.Dotnet.Mocking;

public static class MockingExtensions
{
    public static void InitializeGlobalAppSettingsDefaults()
    {
        GlobalAppSettingsExtensions.Initialize(new NameValueCollection()
        {
            { "UseLocalDataInSystemUserConnection", "true" },
        });
    }
}