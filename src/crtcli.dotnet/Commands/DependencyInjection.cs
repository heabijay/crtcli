using Microsoft.Extensions.DependencyInjection;

namespace CrtCli.Dotnet.Commands;

internal static class DependencyInjection
{
    public static IServiceCollection AddCommands(this IServiceCollection services)
    {
        services.AddTransient<GeneratePackageSchemaSourcesCommand>();

        return services;
    }
}