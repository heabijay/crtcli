using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal static class UpdateSysSchemaLastError
{
    public record Request(Guid SchemaId, string LastError) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText =
                """
                UPDATE "public"."SysSchema"
                SET
                    "LastError" = @P2
                WHERE
                    "Id" = @P1
                """;
            
            if (!DbCommandTextEqualityComparer.Instance.Equals(expectedCommandText, dbCommand.CommandText))
            {
                request = null!;
                return false;
            }

            var lastErrorParameter = (DbParameterMock)dbCommand.Parameters["P2"];
            var lastError = (string)lastErrorParameter.Value!;

            var schemaIdParameter = (DbParameterMock)dbCommand.Parameters["P1"];
            var schemaId = (Guid)schemaIdParameter.Value!;

            request = new Request(schemaId, lastError);
            return true;
        }
    }

    public class Handler(ILogger<Handler> logger) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            logger.LogError("Schema '{SchemaId}' error occured: {Error}", request.SchemaId, request.LastError);

            return new DataTableReader(new DataTable()
            {
                Columns =
                {
                    { "RowsAffected", typeof(int) },
                },
                Rows =
                {
                    { 1 },
                }
            });
        }
    }
}