#if NETSTANDARD

using System;
using System.Linq.Expressions;
using System.Reflection;
using Microsoft.AspNetCore.Http;

namespace CrtCli.Packages.Tunneling.Utilities;

internal static partial class HttpContextUtilities
{
    public static HttpContext GetInnerHttpContext(this Terrasoft.Web.Http.Abstractions.HttpContext abstractHttpContext)
    {
        return s__httpContext.Value.Invoke(abstractHttpContext);
    }
    
    
    private static Lazy<Func<Terrasoft.Web.Http.Abstractions.HttpContext, HttpContext>> s__httpContext 
        = new(_httpContext_BuildExpression);

    private static Func<Terrasoft.Web.Http.Abstractions.HttpContext, HttpContext> _httpContext_BuildExpression()
    {
        var httpContextImplType = Type.GetType("Terrasoft.Web.Http.AspNetCore.HttpContextImpl, Terrasoft.Web.Http.AspNetCore", true);
        
        var thisParameter = Expression.Parameter(typeof(Terrasoft.Web.Http.Abstractions.HttpContext));
        var thisParameterConverted = Expression.Convert(thisParameter, httpContextImplType);
        
        var fieldInfo = httpContextImplType
                .GetField("_httpContext", BindingFlags.FlattenHierarchy | BindingFlags.Instance | BindingFlags.NonPublic)
            ?? throw new ArgumentNullException("_httpContext");
        
        var bodyExpression = Expression.Field(thisParameterConverted, fieldInfo);
        
        var lambda = Expression.Lambda<Func<Terrasoft.Web.Http.Abstractions.HttpContext, HttpContext>>(
            bodyExpression, thisParameter);

        return lambda.Compile();
    }
}

#endif