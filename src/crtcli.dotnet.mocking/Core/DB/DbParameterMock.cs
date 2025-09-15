using System.Data;
using System.Data.Common;
using System.Diagnostics.CodeAnalysis;
using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DbParameterMock : DbParameter
{
    [AllowNull] public override string ParameterName { get; set; }
    
    public override object? Value { get; set; }

    public override DbType DbType
    {
        get => throw new NotImplementedException();
        set => throw new NotImplementedException();
    }
    
    public override ParameterDirection Direction
    {
        get => throw new NotImplementedException();
        set => throw new NotImplementedException();
    }
    
    public override bool IsNullable
    {
        get => throw new NotImplementedException();
        set => throw new NotImplementedException();
    }

    [AllowNull]
    public override string SourceColumn
    {
        get => throw new NotImplementedException();
        set => throw new NotImplementedException();
    }
    
    public override bool SourceColumnNullMapping
    {
        get => throw new NotImplementedException();
        set => throw new NotImplementedException();
    }

    public override int Size
    {
        get => throw new NotImplementedException();
        set => throw new NotImplementedException();
    }
    
    public override void ResetDbType()
    {
        throw new NotImplementedException();
    }
    
    public static implicit operator DbParameterMock(QueryParameter queryParameter)
    {
        return new DbParameterMock()
        {
            ParameterName = queryParameter.Name,
            Value = queryParameter.Value,
        };
    }
    
    public static implicit operator QueryParameter(DbParameterMock dbParameter)
    {
        return new QueryParameter(dbParameter.ParameterName, dbParameter.Value);
    }
}