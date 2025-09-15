using CrtCli.Dotnet.Mocking.Extensions;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace CrtCli.Dotnet.Mocking;

internal static class MockingLogging
{
    // Cannot use proper dependency injection logging providing because of internal mocked classes
    // being initialized with statically defined constructor arguments. 
    // This is workaround. 
    // You can change factory to something like ThreadLocal and try to make it thread-safe.
    public static ILoggerFactory LoggerFactory { get; set; } = 
        CoreApiContainerExtensions
            .get_ServiceProvider()
            .GetService<ILoggerFactory>()
        ?? new NullLoggerFactory();
    
    public static ILogger<T> CreateLogger<T>() => LoggerFactory.CreateLogger<T>();
    
    public static ILogger CreateLogger(string categoryName) => LoggerFactory.CreateLogger(categoryName);
}