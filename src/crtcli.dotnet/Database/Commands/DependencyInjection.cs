using System.Reflection;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using Microsoft.Extensions.DependencyInjection;

namespace CrtCli.Dotnet.Database.Commands;

internal static class DependencyInjection
{
    public static IServiceCollection AddDatabaseCommandMocking(this IServiceCollection services)
    {
        AddRequestParsers(services);
        AddRequestHandlers(services);

        return services;
    }

    private static void AddRequestParsers(this IServiceCollection services)
    {
        var parsersTypes = Assembly
            .GetExecutingAssembly()
            .GetTypes()
            .Where(type => type is { IsClass: true, IsAbstract: false } && typeof(IDbCommandMockRequestParser).IsAssignableFrom(type));

        foreach (var parserType in parsersTypes)
        {
            services.AddSingleton(typeof(IDbCommandMockRequestParser), parserType);
        }
    }

    private static void AddRequestHandlers(this IServiceCollection services)
    {
        var handlersTypes = Assembly
            .GetExecutingAssembly()
            .GetTypes()
            .Where(t => t is { IsClass: true, IsAbstract: false } && typeof(IDbCommandMockRequestHandler).IsAssignableFrom(t));

        foreach (var handlerType in handlersTypes)
        {
            var requestType = handlerType
                .DeclaringType?
                .GetNestedTypes()
                .FirstOrDefault(t => typeof(IDbCommandMockRequest).IsAssignableFrom(t));

            if (requestType is null)
            {
                throw new InvalidOperationException($"Request type not found for handler {handlerType.Name}");
            }

            services.AddTransient(typeof(IDbCommandMockRequestHandler<>).MakeGenericType(requestType), handlerType);
        }
    }
}
