using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal static class LocalizableValuesBySchemaUId
{
    public record Request(Guid SchemaUId) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText = 
            """
            SELECT
                "LocalizableValue"."SysSchemaId" "SchemaId",
                "SysSchema"."UId" "SchemaUId",
                "SysSchema"."Name" "SchemaName",
                "SysPackage"."Name" "PackageName",
                "LocalizableValue"."SysPackageId" "PackageId",
                "LocalizableValue"."SysCultureId" "CultureId",
                "LocalizableValue"."ModifiedOn" "ModifiedOn",
                "LocalizableValue"."Key" "Key",
                "LocalizableValue"."Value" "Value",
                "LocalizableValue"."ResourceType" "ResourceType",
                "LocalizableValue"."ImageData" "ImageData"
            FROM
                "public"."SysLocalizableValue" "LocalizableValue"
                INNER JOIN "public"."SysPackage" ON ("SysPackage"."Id" = "LocalizableValue"."SysPackageId")
                INNER JOIN "public"."SysSchema" ON ("SysSchema"."Id" = "LocalizableValue"."SysSchemaId")
            WHERE
                "SysPackage"."SysWorkspaceId" = @P1
                AND "LocalizableValue"."SysPackageId" IN (
            SELECT
                "SysPackageId"
            FROM
                "public"."SysSchema"
            WHERE
                "Id" IN (@P2))
                AND "LocalizableValue"."SysSchemaId" = @P3
            """;

            if (!DbCommandTextEqualityComparer.Instance.Equals(dbCommand.CommandText, expectedCommandText))
            {
                request = null!;
                return false;
            }

            var schemaUIdParameter = (DbParameterMock)dbCommand.Parameters["P3"];
            var schemaUId = (Guid)schemaUIdParameter.Value!;

            request = new Request(schemaUId);
            return true;
        }
    }

    public class Handler(ILogger<Handler> logger) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            logger.LogDebug("LocalizableValuesBySchemaUId handler for SchemaUId {SchemaUId} is returning no rows", request.SchemaUId);

            return new DataTableReader(new DataTable());
        }
    }
}