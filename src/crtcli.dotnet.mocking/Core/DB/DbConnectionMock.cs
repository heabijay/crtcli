using System.Data;
using System.Data.Common;
using System.Diagnostics.CodeAnalysis;
using CrtCli.Dotnet.Mocking.Extensions;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DbConnectionMock : DbConnection
{
    private readonly ILogger<DbConnectionMock> _logger = MockingLogging.CreateLogger<DbConnectionMock>();
    private ConnectionState _state = ConnectionState.Closed;

    private const string DefaultDatabaseName = "creatio";

    protected override DbCommand CreateDbCommand() =>
        CoreApiContainerExtensions
            .get_ServiceProvider()
            .GetRequiredService<IDbCommandFactory>()
            .CreateDbCommand();

    protected override DbTransaction BeginDbTransaction(IsolationLevel isolationLevel)
    {
        throw new NotImplementedException();
    }

    public override void ChangeDatabase(string databaseName)
    {
        throw new NotImplementedException();
    }

    public override void Open()
    {
        _state = ConnectionState.Open;
    }

    public override void Close()
    {
        _state = ConnectionState.Closed;
    }

    [AllowNull]
    public override string ConnectionString { get; set; }

    public override string Database => DefaultDatabaseName;

    public override ConnectionState State => _state;

    public override string DataSource => throw new NotImplementedException();

    public override string ServerVersion => throw new NotImplementedException();
}