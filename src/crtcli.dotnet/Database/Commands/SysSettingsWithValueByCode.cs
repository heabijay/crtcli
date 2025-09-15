using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal static class SysSettingsWithValueByCode
{
    public record Request(string Code) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText =
                """
                SELECT
                    "Code",
                    "ValueTypeName",
                    "IsCacheable",
                    "Position",
                    "SysAdminUnitId",
                    "TextValue",
                    "IntegerValue",
                    "FloatValue",
                    "BooleanValue",
                    "DateTimeValue",
                    "GuidValue",
                    "BinaryValue"
                FROM
                    "public"."SysSettings"
                    LEFT OUTER JOIN "public"."SysSettingsValue" ON ("SysSettings"."Id" = "SysSettingsValue"."SysSettingsId")
                WHERE
                    "SysSettings"."Code" = @P1
                ORDER BY
                    "Position" ASC
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
            logger.LogDebug("No SysSettings & SysSettingsValue was returned for code {Code}, expecting the system should use default value instead", request.Code);
            
            return new DataTableReader(new DataTable());
        }
    }
}
