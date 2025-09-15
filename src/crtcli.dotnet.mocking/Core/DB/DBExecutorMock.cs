using System.Data.Common;
using System.Diagnostics.CodeAnalysis;
using CrtCli.Dotnet.Mocking.Extensions;
using Microsoft.Extensions.DependencyInjection;
using Terrasoft.Common;
using Terrasoft.Core;
using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DBExecutorMock : DBExecutor
{
    public DBExecutorMock(UserConnection userConnection) : base(userConnection)
    {
    }

    public DBExecutorMock(UserConnection userConnection, QueryKind queryKind) : base(userConnection, queryKind)
    {
    }

    protected override DbConnection NewConnection() => new DbConnectionMock();

    protected override DbCommand NewCommand() =>
        CoreApiContainerExtensions
            .get_ServiceProvider()
            .GetRequiredService<IDbCommandFactory>()
            .CreateDbCommand();

    protected override string BuildConnectionString(string userName, string password)
    {
        throw new NotImplementedException();
    }

    protected override void InitilizeConnectionAfterOpen(DbConnection dbConnection)
    {
    }

    protected override void QueryParametersToDBParameters(
        QueryParameterCollection? queryParameters,
        DbParameterCollection? dbParameters)
    {
        if (queryParameters is null || dbParameters is null)
        {
            return;
        }

        foreach (var queryParameter in queryParameters)
        {
            dbParameters.Add((DbParameterMock)queryParameter);
        }
    }

    protected override void DBParametersToQueryParameters(
        DbParameterCollection? dbParameters,
        QueryParameterCollection? queryParameters)
    {
        if (dbParameters is null || queryParameters is null)
        {
            return;
        }

        foreach (DbParameterMock dbParameter in dbParameters)
        {
            queryParameters.Add(dbParameter);
        }
    }

    protected override IEnumerable<string> SplitBatches(string sqlText)
    {
        throw new NotImplementedException();
    }

    protected override bool ValidateBatches(DbCommand command, string sqlText, [UnscopedRef] out string message)
    {
        throw new NotImplementedException();
    }
}