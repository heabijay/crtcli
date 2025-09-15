using Terrasoft.Core;
using Terrasoft.Core.DB;
using Terrasoft.Core.Process;

namespace CrtCli.Dotnet.Mocking.Core;

public class ManagerProviderSectionCollectionMock : ManagerProviderSectionCollection
{
    public ManagerProviderSectionCollectionMock(IEnumerable<ManagerProviderConfigurationElement> elements)
    {
        foreach (var element in elements)
        {
            // ReSharper disable once VirtualMemberCallInConstructor
            BaseAdd(element);
        }
    }
    
    public static ManagerProviderSectionCollectionMock CreateDefault()
    {
        return new ManagerProviderSectionCollectionMock([
            new ManagerProviderConfigurationElementMock(
                name: "DataValueTypeManager", 
                type: typeof(DataValueTypeManager),
                scope: ManagerScope.app),
            
            new ManagerProviderConfigurationElementMock(
                name: "SystemValueManager", 
                type: typeof(SystemValueManager),
                scope: ManagerScope.app),
            
            // ----------------------------------------------------------------------------- //
            
            new ManagerProviderConfigurationElementMock(
                name: "LicManager", 
                type: typeof(LicManager),
                scope: ManagerScope.app),
            
            new ManagerProviderConfigurationElementMock(
                name: "DesignModeValuesProviderManager", 
                type: typeof(DesignModeValuesProviderManager),
                scope: ManagerScope.app),
            
            new ManagerProviderConfigurationElementMock(
                name: "ProcessSchemaElementManager", 
                type: typeof(ProcessSchemaElementManager),
                scope: ManagerScope.user),
            
            new ManagerProviderConfigurationElementMock(
                name: "DBMetaActionManager", 
                type: typeof(DBMetaActionManager),
                scope: ManagerScope.user),
        ]);
    }
}