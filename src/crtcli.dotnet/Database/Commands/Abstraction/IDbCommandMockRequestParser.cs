using System.Data;

namespace CrtCli.Dotnet.Database.Commands.Abstraction;

internal interface IDbCommandMockRequestParser<TRequest> : IDbCommandMockRequestParser
    where TRequest : IDbCommandMockRequest
{
    bool TryParse(IDbCommand dbCommand, out TRequest request);

    bool IDbCommandMockRequestParser.TryParse(IDbCommand dbCommand, out IDbCommandMockRequest request)
    {
        if (TryParse(dbCommand, out TRequest requestCasted))
        {
            request = requestCasted;
            return true;
        }
        
        request = null!;
        return false;
    }
}

internal interface IDbCommandMockRequestParser
{
    bool TryParse(IDbCommand dbCommand, out IDbCommandMockRequest request);
}