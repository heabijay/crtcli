using System.Configuration;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Core;

public class SchemaManagerProviderConfigurationElementMock(
    string name,
    Type type,
    string? compileDependencies = null) 
    : SchemaManagerProviderConfigurationElement
{
    protected override ConfigurationPropertyCollection Properties { get; } =
    [
        new ConfigurationProperty(
            "name",
            typeof(string), 
            name, 
            ConfigurationPropertyOptions.IsRequired),
        
        new ConfigurationProperty(
            "type",
            typeof(Type),
            type,
            ConfigurationPropertyOptions.IsRequired),
        
        new ConfigurationProperty(
            "compileDependencies",
            typeof(string),
            compileDependencies,
            ConfigurationPropertyOptions.IsRequired)
    ];
}