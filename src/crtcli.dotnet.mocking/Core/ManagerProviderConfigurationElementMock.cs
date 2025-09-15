using System.Configuration;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Core;

public class ManagerProviderConfigurationElementMock(
    string name,
    Type type,
    ManagerScope scope) 
    : ManagerProviderConfigurationElement
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
            "scope",
            typeof(ManagerScope),
            scope,
            ConfigurationPropertyOptions.IsRequired)
    ];
}