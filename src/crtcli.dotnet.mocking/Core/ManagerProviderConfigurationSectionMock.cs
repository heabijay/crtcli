using System.Configuration;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Core;

public class ManagerProviderConfigurationSectionMock : ManagerProviderConfigurationSection
{
    private static readonly ConfigurationProperty ManagerProvidersProperty = new(
        string.Empty,
        typeof(ManagerProviderSectionCollection),
        null,
        ConfigurationPropertyOptions.IsDefaultCollection | ConfigurationPropertyOptions.IsRequired);

    
    public ManagerProviderConfigurationSectionMock(ManagerProviderSectionCollection sectionCollection)
    {
        this[ManagerProvidersProperty] = sectionCollection;
    }
}