using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal class HierarchicalSelect
{
    public record Request(Guid SchemaUId) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText = 
                """
                SELECT
                    "Id",
                    "Name",
                    "ParentId"
                FROM
                    "$HierarchicalSelect"
                """;

            if (!DbCommandTextEqualityComparer.Instance.Equals(dbCommand.CommandText, expectedCommandText))
            {
                request = null!;
                return false;
            }

            var schemaUIdParameter = (DbParameterMock)dbCommand.Parameters["SchemaUId"];
            var schemaUId = (Guid)schemaUIdParameter.Value!;

            request = new Request(schemaUId);
            return true;
        }
    }

    public class Handler(ILogger<Handler> logger) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            logger.LogDebug("Invocation of HierarchicalSelect Handler for schemaUId {SchemaUId}, returning no rows", request.SchemaUId);
            
            return new DataTableReader(new DataTable());
        }
    }
}
