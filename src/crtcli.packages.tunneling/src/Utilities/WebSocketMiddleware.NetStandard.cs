#if NETSTANDARD

using System;
using System.Linq.Expressions;
using System.Reflection;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;

namespace CrtCli.Packages.Tunneling.Utilities;

internal static partial class WebSocketMiddleware
{
    public static readonly Type Type = GetWebSocketMiddlewareType();

    public static object CreateInstance(IServiceProvider serviceProvider, RequestDelegate requestDelegate)
    {
        return WebSocketMiddlewareObjectFactory.Value.Invoke(serviceProvider, [requestDelegate]);
    }
    
    public static Task Invoke(object webSocketMiddleware, HttpContext httpContext)
    {
        return s_Invoke.Value.Invoke(webSocketMiddleware, httpContext);
    }
    

    private static readonly Lazy<ObjectFactory> WebSocketMiddlewareObjectFactory = 
        new(() => ActivatorUtilities.CreateFactory(Type, [typeof(RequestDelegate)]));
        
    private static Type GetWebSocketMiddlewareType()
    {
        return Type.GetType("Microsoft.AspNetCore.WebSockets.WebSocketMiddleware, Microsoft.AspNetCore.WebSockets", true);
    }
    
    
    private static Lazy<Func<object, HttpContext, Task>> s_Invoke = new(Invoke_BuildExpression);

    private static Func<object, HttpContext, Task> Invoke_BuildExpression()
    {
        var thisParameter = Expression.Parameter(typeof(object));
        var thisParameterConverted = Expression.Convert(thisParameter, Type);
        var httpContext = Expression.Parameter(typeof(HttpContext));

        var methodInfo = Type.GetMethod(
                "Invoke",
                BindingFlags.Public | BindingFlags.Instance,
                null,
                [typeof(HttpContext)],
                null)
            ?? throw new ArgumentNullException("WebSocketMiddleware.Invoke");
        
        var bodyExpression = Expression.Call(thisParameterConverted, methodInfo, httpContext);
        
        var lambda = Expression.Lambda<Func<object, HttpContext, Task>>(
            bodyExpression, thisParameter, httpContext);

        return lambda.Compile();
    }
}

#endif