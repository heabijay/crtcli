using CrtCli.Dotnet.Mocking.Core.Entities;
using CrtCli.Dotnet.Mocking.Core.Process;
using Terrasoft.Core;
using Terrasoft.Core.Campaign;
using Terrasoft.Core.DcmProcess;

namespace CrtCli.Dotnet.Mocking.Core;

public class SchemaManagerProviderSectionCollectionMock : SchemaManagerProviderSectionCollection
{
    public SchemaManagerProviderSectionCollectionMock(IEnumerable<SchemaManagerProviderConfigurationElement> elements)
    {
        foreach (var element in elements)
        {
            // ReSharper disable once VirtualMemberCallInConstructor
            BaseAdd(element);
        }
    }

    public static SchemaManagerProviderSectionCollectionMock CreateDefault()
    {
        return new SchemaManagerProviderSectionCollectionMock([
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "EntitySchemaManager", 
                type: typeof(EntitySchemaManagerMock)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "ProcessUserTaskSchemaManager", 
                type: typeof(ProcessUserTaskSchemaManagerMock)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "ProcessSchemaManager", 
                type: typeof(ProcessSchemaManagerMock)),
            
            // ----------------------------------------------------------------------------- //
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "DcmSchemaManager", 
                type: typeof(DcmSchemaManager)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "CampaignSchemaManager", 
                type: typeof(CampaignSchemaManager)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "ImageListSchemaManager", 
                type: typeof(ImageListSchemaManager)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "SourceCodeSchemaManager", 
                type: typeof(SourceCodeSchemaManager)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "ValueListSchemaManager", 
                type: typeof(ValueListSchemaManager)),
            
            new SchemaManagerProviderConfigurationElementMock(
                name: "ClientUnitSchemaManager", 
                type: typeof(ClientUnitSchemaManager)),
            
            // new SchemaManagerProviderConfigurationElementMock(
            //     name: "AddonSchemaManager", 
            //     type: typeof(AddonSchemaManager)),
            
            // new SchemaManagerProviderConfigurationElementMock(
            //     name: "ServiceSchemaManager", 
            //     type: Type.GetType("Terrasoft.Services.ServiceSchemaManager, Terrasoft.Services")!),
        ]);
    }
}