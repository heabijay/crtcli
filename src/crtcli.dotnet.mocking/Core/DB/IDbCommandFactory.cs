using System.Data.Common;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public interface IDbCommandFactory
{
    public DbCommand CreateDbCommand();
}