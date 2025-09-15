using CrtCli.Dotnet.Database.Commands;
using CrtCli.Dotnet.Mocking.Core.DB;
using Microsoft.Extensions.DependencyInjection;

namespace CrtCli.Dotnet.Database;

public static class DependencyInjection
{
    public static IServiceCollection AddDatabaseMocking(this IServiceCollection services)
    {
        services.AddTransient<IDbCommandMockHandler, DbCommandMockHandler>();
        services.AddTransient<IDbCommandFactory, DbCommandMockFactory>();
        
        services.AddDatabaseCommandMocking();

        return services;
    }
}