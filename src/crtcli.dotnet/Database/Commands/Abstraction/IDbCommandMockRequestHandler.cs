using System.Data.Common;

namespace CrtCli.Dotnet.Database.Commands.Abstraction;

internal interface IDbCommandMockRequestHandler<in TRequest> : IDbCommandMockRequestHandler
    where TRequest : IDbCommandMockRequest
{
    DbDataReader Handle(TRequest request);
    
    DbDataReader IDbCommandMockRequestHandler.Handle(IDbCommandMockRequest request)
    {
        return Handle((TRequest)request);
    }
}

internal interface IDbCommandMockRequestHandler
{
    DbDataReader Handle(IDbCommandMockRequest request);
}