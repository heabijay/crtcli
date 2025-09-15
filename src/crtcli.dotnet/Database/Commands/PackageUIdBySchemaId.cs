using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Context;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;

namespace CrtCli.Dotnet.Database.Commands;

internal static class PackageUIdBySchemaId
{
    public record Request(Guid SchemaId) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText = 
            """
            SELECT
                "PackageUId"
            FROM
                "public"."VwSysSchemaInWorkspace"
            WHERE
                "SysWorkspaceId" = @P1
                AND "Id" = @P2
            """;

            if (!DbCommandTextEqualityComparer.Instance.Equals(dbCommand.CommandText, expectedCommandText))
            {
                request = null!;
                return false;
            }

            var schemaIdParameter = (DbParameterMock)dbCommand.Parameters["P2"];
            var schemaId = (Guid)schemaIdParameter.Value!;

            request = new Request(schemaId);
            return true;
        }
    }

    public class Handler(IPackageContext packageContext) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            // Because we use schemaId as the same as schemaUId, we can query by it

            if (packageContext.Schemas.All(x => x.Descriptor.UId != request.SchemaId))
            {
                throw new CannotFindSchemaInContextException(request.SchemaId);
            }
            
            return new DataTableReader(new DataTable()
            {
                Columns = 
                { 
                    { "PackageUId", typeof(Guid) } 
                },
                Rows =
                {
                    { packageContext.Descriptor.UId }
                }
            });
        }
    }

    public class CannotFindSchemaInContextException(Guid schemaId) 
        : Exception($"Cannot find schema with Id '{schemaId}' in current context")
    {
        public Guid SchemaId { get; } = schemaId;
    }
}
