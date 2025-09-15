using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal static class SysSettingsByCode
{
    public record Request(string Code) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText =
                """
                SELECT
                    "Id",
                    "Name",
                    "Code",
                    "Description",
                    "ValueTypeName",
                    "IsPersonal",
                    "IsCacheable",
                    "IsSSPAvailable",
                    "ReferenceSchemaUId"
                FROM
                    "public"."SysSettings"
                WHERE
                    "Code" = @P1
                """;
            
            if (!DbCommandTextEqualityComparer.Instance.Equals(expectedCommandText, dbCommand.CommandText))
            {
                request = null!;
                return false;
            }

            var codeParameter = (DbParameterMock)dbCommand.Parameters["P1"];
            var code = (string)codeParameter.Value!;

            request = new Request(code);
            return true;
        }
    }

    public class Handler(ILogger<Handler> logger) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            logger.LogDebug("No SysSettings was returned for code {Code}, expecting the system should use default value instead", request.Code);

            return new DataTableReader(new DataTable());
        }
    }
}
