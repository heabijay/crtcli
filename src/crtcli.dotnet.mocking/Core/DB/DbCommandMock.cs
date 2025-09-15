using System.Data;
using System.Data.Common;
using System.Diagnostics.CodeAnalysis;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DbCommandMock(IDbCommandMockHandler dbCommandMockHandler) : DbCommand
{
    private readonly ILogger<DbCommandMock> _logger = MockingLogging.CreateLogger<DbCommandMock>();

    [AllowNull] public override string CommandText { get; set; }

    public override int CommandTimeout { get; set; } = 0;

    public override CommandType CommandType { get; set; } = CommandType.Text;

    public override UpdateRowSource UpdatedRowSource { get; set; } = UpdateRowSource.FirstReturnedRecord;
    
    protected override DbConnection? DbConnection { get; set; }

    protected override DbParameterCollection DbParameterCollection { get; } = new DbParameterCollectionMock();
    
    protected override DbTransaction? DbTransaction { get; set; }
    
    public override bool DesignTimeVisible { get; set; }
    
    public override void Prepare()
    {
        _logger.LogTrace("`DbCommandMock.Prepare` was called, but no action was taken");
    }
    
    public override void Cancel()
    {
        _logger.LogTrace("`DbCommandMock.Cancel` was called, but no action was taken");
    }

    protected override DbParameter CreateDbParameter()
    {
        throw new NotImplementedException();
    }

    public override int ExecuteNonQuery()
    {
        _logger.LogTrace("Executing non-query DbCommand: {CommandText}", this.CommandText);
        
        using var reader = dbCommandMockHandler.Handle(this, DbCommandQueryType.ExecuteNonQuery);
        
        if (reader.Read())
        {
            return (int)reader.GetValue(0);
        }
        
        return 0;
    }

    public override object? ExecuteScalar()
    {
        _logger.LogTrace("Executing scalar DbCommand: {CommandText}", this.CommandText);
        
        using var reader = dbCommandMockHandler.Handle(this, DbCommandQueryType.ExecuteScalar);
        
        if (reader.Read())
        {
            return reader.GetValue(0);
        }
        
        return null;
    }

    protected override DbDataReader ExecuteDbDataReader(CommandBehavior behavior)
    {
        _logger.LogTrace("Executing DbCommand: {CommandText}", this.CommandText);
        
        return dbCommandMockHandler.Handle(this, DbCommandQueryType.ExecuteReader);
    }
}