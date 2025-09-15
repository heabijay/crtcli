using CrtCli.Dotnet.Mocking.Core;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Mocking.Core.Packages;
using CrtCli.Dotnet.Mocking.Core.Process;
using CrtCli.Dotnet.Mocking.Extensions;
using Microsoft.Extensions.DependencyInjection;
using Terrasoft.Core;
using Terrasoft.Core.Configuration;
using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Mocking;

public static class MockingDependencyInjection
{
    public static IServiceCollection AddCrtCliDotnetMocking(this IServiceCollection services)
    {
        services.AddSingleton<DBEngine, DBEngineMock>();
        services.AddSingleton<DBTypeConverter, DBTypeConverterMock>();
        services.AddSingleton<SchemaManagerProviderConfigurationSection>(
            new SchemaManagerProviderConfigurationSectionMock(
                SchemaManagerProviderSectionCollectionMock.CreateDefault()));
        services.AddSingleton<ManagerProviderConfigurationSection>(
            new ManagerProviderConfigurationSectionMock(
                ManagerProviderSectionCollectionMock.CreateDefault()));
        services.AddSingleton<AppConnection>(AppConnectionFactory);
        services.AddSingleton<SystemUserConnection>(SystemUserConnectionFactory);
        services.AddSingleton<UserConnection>(sp => sp.GetRequiredService<SystemUserConnection>());
        
        services.AddTransient<IProcessEngine, ProcessEngineImplMock>();
        
        // services.AddTransient<IPackageTypeProviderMock, ...>();
        
        services.AddScoped(PackageTypeProviderMock.InterfaceType, PackageTypeProviderFactory);
        
        return services;
    }

    private static AppConnection AppConnectionFactory(IServiceProvider serviceProvider)
    {
        var appConnection = new AppConnection();

        // TODO: Maybe it will be possible to use the Initialize() method
        //       instead of manual, particular initialization, in the future?
        
        appConnection.SystemUserConnection.DBEngine = serviceProvider.GetRequiredService<DBEngine>();
        appConnection.SystemUserConnection.DBExecutorType = typeof(DBExecutorMock);
        appConnection.SystemUserConnection.DBTypeConverter = serviceProvider.GetRequiredService<DBTypeConverter>();
        appConnection.SystemUserConnection.set_DBSecurityEngine(new SystemDBSecurityEngine());
        appConnection.Workspace = new SysWorkspace(appConnection.SystemUserConnection);
        appConnection.Workspace.ResourceStorage = new SchemaResourceStorage(appConnection.Workspace);
        
        appConnection.Workspace.SchemaManagerProvider.Initialize(
            serviceProvider.GetRequiredService<SchemaManagerProviderConfigurationSection>());
        
        appConnection.set_AppManagerProvider(new ManagerProviderMock(
            appConnection, 
            serviceProvider.GetRequiredService<ManagerProviderConfigurationSection>()));
        
        appConnection.SystemUserConnection.set_CurrentUser(new SysUserInfo(appConnection.SystemUserConnection)
        {
            Id = MockingConstants.Supervisor.User.Id,
            Name = MockingConstants.Supervisor.User.Name,
            ContactId = MockingConstants.Supervisor.Contact.Id,
            ContactName = MockingConstants.Supervisor.Contact.Name,
        });
        
        foreach (var columnValue in appConnection.Workspace.get_ColumnValues())
        {
            columnValue.IsLoaded = true;

            if (columnValue.Name == "Id")
            {
                columnValue.Value = Guid.NewGuid();
            }
        }
        
        appConnection.SystemUserConnection.DBSecurityEngine.Initialize(appConnection.SystemUserConnection);

        return appConnection;
    }

    private static SystemUserConnection SystemUserConnectionFactory(IServiceProvider serviceProvider)
    {
        return (SystemUserConnection)serviceProvider.GetRequiredService<AppConnection>().SystemUserConnection;
    }

    private static object PackageTypeProviderFactory(IServiceProvider serviceProvider)
    {
        return PackageTypeProviderMock.CreateNew(
            serviceProvider.GetRequiredService<IPackageTypeProviderMock>());
    }
}