using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Utilities;

namespace CrtCli.Dotnet.Database.Commands;

internal static class SysCulture
{
    public record struct Request : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText = 
                """
                SELECT
                    "Id",
                    "Name",
                    "Active"
                FROM
                    "public"."SysCulture"
                """;

            if (!DbCommandTextEqualityComparer.Instance.Equals(expectedCommandText, dbCommand.CommandText))
            {
                request = default;
                return false;
            }

            request = new Request();
            return true;
        }
    }

    public class Handler : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            return new DataTableReader(new DataTable()
            {
                Columns =
                {
                    { "Id", typeof(Guid) },
                    { "Name", typeof(string) },
                    { "Active", typeof(bool) },
                },
                Rows =
                {
                    { new Guid("a5420246-0a8e-e111-84a3-00155d054c03"), "en-US", true },
                }
            });
        }
    }
}
