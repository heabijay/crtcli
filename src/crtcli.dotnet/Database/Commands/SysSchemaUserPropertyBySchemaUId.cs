using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal static class SysSchemaUserPropertyBySchemaUId
{
    public record Request(Guid SchemaUId, string ManagerName) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            var check1 = DbCommandTextEqualityComparer.Instance.StartsWith(
                dbCommand.CommandText, 
                """
                SELECT
                    "SysSchemaUserProperty"."Name" "Name",
                    "SysSchemaUserProperty"."Value" "Value"
                """);

            var check2 = DbCommandTextEqualityComparer.Instance.Contains(
                dbCommand.CommandText, 
                """
                FROM
                    "public"."SysSchemaUserProperty" "SysSchemaUserProperty"
                """);
            
            var check3 = DbCommandTextEqualityComparer.Instance.Contains(
                dbCommand.CommandText, 
                """
                SELECT
                    "SysSchema"."Id" "Id"
                FROM
                    "public"."SysSchema" "SysSchema"
                    LEFT OUTER JOIN "public"."SysPackage" "SysPackage" ON ("SysPackage"."Id" = "SysSchema"."SysPackageId")
                WHERE
                    "SysSchemaUserProperty"."SysSchemaId" = "SysSchema"."Id"
                    AND ("SysSchema"."ManagerName" = @P1
                    AND "SysPackage"."SysWorkspaceId" = @P2
                    AND "SysSchema"."UId" = @P3)
                """);

            if (!check1 || !check2 || !check3)
            {
                request = null!;
                return false;
            }

            var schemaUIdParameter = (DbParameterMock)dbCommand.Parameters["P3"];
            var schemaUId = (Guid)schemaUIdParameter.Value!;

            var managerNameParameter = (DbParameterMock)dbCommand.Parameters["P1"];
            var managerName = (string)managerNameParameter.Value!;

            request = new Request(schemaUId, managerName);
            return true;
        }
    }

    public class Handler(ILogger<Handler> logger) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            logger.LogDebug("Returning no rows for SysSchemaUserProperty query for schemaUId {SchemaUId} and managerName {ManagerName}", request.SchemaUId, request.ManagerName);

            return new DataTableReader(new DataTable());
        }
    }
}