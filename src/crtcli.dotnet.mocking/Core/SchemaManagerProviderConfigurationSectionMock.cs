using System.Configuration;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Core;

public class SchemaManagerProviderConfigurationSectionMock : SchemaManagerProviderConfigurationSection
{
    private static readonly ConfigurationProperty SchemaManagerProvidersProperty = new(
        string.Empty,
        typeof(SchemaManagerProviderSectionCollection),
        null,
        ConfigurationPropertyOptions.IsDefaultCollection | ConfigurationPropertyOptions.IsRequired);

    
    public SchemaManagerProviderConfigurationSectionMock(SchemaManagerProviderSectionCollection sectionCollection)
    {
        this[SchemaManagerProvidersProperty] = sectionCollection;
    }
}