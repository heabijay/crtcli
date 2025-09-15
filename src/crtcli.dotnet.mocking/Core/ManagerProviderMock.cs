using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Core;

public class ManagerProviderMock : ManagerProvider
{
    public ManagerProviderMock(
        AppConnection appConnection,
        ManagerProviderConfigurationSection settings) 
        : base(appConnection)
    {
        Settings = settings;
    }
}