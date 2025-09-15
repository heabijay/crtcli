using System.Linq.Expressions;
using System.Reflection;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class CoreApiContainerExtensions
{
    private static readonly Type CoreApiContainerType =
        Type.GetType("Terrasoft.Core.DI.CoreApiContainer, Terrasoft.Core.DI", true)!;
    
    private static readonly MethodInfo InitMethod =
        CoreApiContainerType.GetMethod(
            "Init",
            BindingFlags.Static | BindingFlags.NonPublic)
        ?? throw new MissingMethodException(CoreApiContainerType.FullName, "Init");
    
    
    public static void Initialize(IServiceProvider serviceProvider)
    {
        ArgumentNullException.ThrowIfNull(serviceProvider);

        InitMethod.Invoke(null, [serviceProvider]);
    }

    public static IServiceProvider get_ServiceProvider() => get_ServiceProvider_Lambda.Value.Invoke();

    private static Lazy<Func<IServiceProvider>> get_ServiceProvider_Lambda => new(get_ServiceProvider_CompileLambda);

    private static Func<IServiceProvider> get_ServiceProvider_CompileLambda()
    {
        var serviceProviderProp = Expression.Property(null, CoreApiContainerType, "ServiceProvider");
        
        return Expression.Lambda<Func<IServiceProvider>>(serviceProviderProp).Compile();
    }
}