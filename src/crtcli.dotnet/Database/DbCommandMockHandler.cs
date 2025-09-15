using System.Collections.Concurrent;
using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database;

internal class DbCommandMockHandler(
    ILogger<DbCommandMockHandler> logger,
    IEnumerable<IDbCommandMockRequestParser> dbCommandRequestParsers,
    IServiceProvider serviceProvider)
    : IDbCommandMockHandler
{
    private static readonly ConcurrentDictionary<Type, Type> HandlerTypeCache = new();
    
    public DbDataReader Handle(IDbCommand dbCommand, DbCommandQueryType queryType)
    {
        var request = ParseCommand(dbCommand);
        
        if (request == null)
        {
            throw new DbCommandMockHandlerNotFoundException(dbCommand, queryType);
        }
        
        var requestType = request.GetType();

        if (logger.IsEnabled(LogLevel.Debug))
        {
            logger.LogDebug(
                "Executing DbCommand mock {RequestTypeName}+{Request}", 
                requestType.DeclaringType?.Name ?? "", 
                request);
        }
        
        return GetHandler(requestType, request).Handle(request);
    }
    
    private IDbCommandMockRequest? ParseCommand(IDbCommand dbCommand)
    {
        foreach (var parser in dbCommandRequestParsers)
        {
            if (parser.TryParse(dbCommand, out var request))
            {
                return request;
            }
        }
        
        return null;
    }

    private IDbCommandMockRequestHandler GetHandler(Type requestType, IDbCommandMockRequest request)
    {
        var handlerType = HandlerTypeCache.GetOrAdd(
            requestType, 
            static x => typeof(IDbCommandMockRequestHandler<>).MakeGenericType(x));
        
        return (IDbCommandMockRequestHandler)serviceProvider.GetRequiredService(handlerType);
    }
    
    
    [Serializable]
    internal class DbCommandMockHandlerNotFoundException(
        IDbCommand dbCommand, 
        DbCommandQueryType queryType)
        : ApplicationException($"Concrete DbCommandMockHandler was not found for QueryType {queryType} with CommandText: {dbCommand.CommandText}")
    {
        public IDbCommand DbCommand { get; } = dbCommand;

        public DbCommandQueryType QueryType { get; } = queryType;
    }
}