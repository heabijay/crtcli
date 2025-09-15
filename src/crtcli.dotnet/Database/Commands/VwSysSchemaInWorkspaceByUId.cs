using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Context;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Database.Commands;

internal static class VwSysSchemaInWorkspaceByUId
{
    public record Request(Guid SchemaUId, string ManagerName) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText =
                """
                SELECT
                    "Id",
                    "UId",
                    "Name",
                    "ManagerName",
                    "MetaData"
                FROM
                    "public"."VwSysSchemaInWorkspace"
                WHERE
                    "UId" = @SchemaUId
                    AND "ManagerName" = @P1
                    AND "VwSysSchemaInWorkspace"."SysWorkspaceId" = @P2
                """;
            
            if (!DbCommandTextEqualityComparer.Instance.Equals(expectedCommandText, dbCommand.CommandText))
            {
                request = null!;
                return false;
            }

            var schemaUIdParameter = (DbParameterMock)dbCommand.Parameters["SchemaUId"];
            var schemaUId = (Guid)schemaUIdParameter.Value!;

            var managerNameParameter = (DbParameterMock)dbCommand.Parameters["P1"];
            var managerName = (string)managerNameParameter.Value!;

            request = new Request(schemaUId, managerName);
            return true;
        }
    }

    public class Handler(
        IPackageContext packageContext,
        AppConnection appConnection)
        : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            var schemaContext = packageContext
                .Schemas
                .FirstOrDefault(x => x.Descriptor.UId == request.SchemaUId);

            if (schemaContext is null)
            {
                if (request.ManagerName == "ProcessSchemaManager"
                    && request.SchemaUId == appConnection.Workspace.SchemaManagerProvider.GetManager("ProcessSchemaManager").GetDefSchemaUId())
                {
                    // All is ok, no operation is required for Process schema
                    return new DataTableReader(new DataTable());
                }
                else
                {
                    throw new CannotFindSchemaByUIdInContextException(request.SchemaUId);
                }
            }

            return new DataTableReader(new DataTable()
            {
                Columns =
                {
                    { "Id", typeof(Guid) },
                    { "UId", typeof(Guid) },
                    { "Name", typeof(string) },
                    { "ManagerName", typeof(string) },
                    { "MetaData", typeof(byte[]) },
                },
                Rows =
                {
                    { 
                        schemaContext.Descriptor.UId, 
                        schemaContext.Descriptor.UId, 
                        schemaContext.Descriptor.Name, 
                        schemaContext.Descriptor.ManagerName, 
                        schemaContext.GetMetadata() 
                    },
                }
            });
        }
    }

    public class CannotFindSchemaByUIdInContextException(Guid schemaUId) 
        : Exception($"Cannot find schema {schemaUId} in current context")
    {
        public Guid SchemaUId { get; } = schemaUId;
    }
}
