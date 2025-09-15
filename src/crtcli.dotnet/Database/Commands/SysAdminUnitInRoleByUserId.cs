using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Database.Commands;

internal static class SysAdminUnitInRoleByUserId
{
    public record Request(Guid UserId, SysAdminUnitType[] ExcludeRoleTypes) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText = 
                """
                SELECT
                    "User"."Id" "UserId",
                    "User"."ConnectionType" "ConnectionType",
                    "SysAdminUnitInRole"."SysAdminUnitRoleId" "RoleId"
                FROM
                    "public"."SysAdminUnitInRole"
                    INNER JOIN "public"."SysAdminUnit" "User" ON ("SysAdminUnitInRole"."SysAdminUnitId" = "User"."Id")
                    INNER JOIN "public"."SysAdminUnit" "Role" ON ("SysAdminUnitInRole"."SysAdminUnitRoleId" = "Role"."Id")
                WHERE
                    "SysAdminUnitInRole"."SysAdminUnitId" = @P1
                    AND "Role"."SysAdminUnitTypeValue" <> @P2
                    AND "Role"."SysAdminUnitTypeValue" <> @P3
                """;

            if (!DbCommandTextEqualityComparer.Instance.Equals(dbCommand.CommandText, expectedCommandText))
            {
                request = null!;
                return false;
            }

            var userIdParameter = (DbParameterMock)dbCommand.Parameters["P1"];
            var userId = (Guid)userIdParameter.Value!;

            var excludeRoleTypesParameter = (DbParameterMock)dbCommand.Parameters["P2"];
            var excludeRoleTypes = (SysAdminUnitType)excludeRoleTypesParameter.Value!;

            var excludeRoleTypesParameter2 = (DbParameterMock)dbCommand.Parameters["P3"];
            var excludeRoleTypes2 = (SysAdminUnitType)excludeRoleTypesParameter2.Value!;

            request = new Request(userId, [excludeRoleTypes, excludeRoleTypes2]);
            return true;
        }
    }

    public class Handler : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            if (request.UserId != MockingConstants.Supervisor.User.Id
                || request.ExcludeRoleTypes is not [SysAdminUnitType.User, SysAdminUnitType.SelfServicePortalUser])
            {
            }

            return new DataTableReader(new DataTable()
            {
                Columns =
                {
                    { "UserId", typeof(Guid) },
                    { "ConnectionType", typeof(int) },
                    { "RoleId", typeof(Guid) },
                },
                Rows =
                {
                    { MockingConstants.Supervisor.User.Id, 0, new Guid("83a43ebc-f36b-1410-298d-001e8c82bcad") },
                    { MockingConstants.Supervisor.User.Id, 0, new Guid("a29a3ba5-4b0d-de11-9a51-005056c00008") },
                }
            });
        }
    }
}