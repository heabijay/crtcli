using System.Data.Common;
using CrtCli.Dotnet.Mocking.Core.DB;

namespace CrtCli.Dotnet.Database;

internal class DbCommandMockFactory(IDbCommandMockHandler dbCommandMockHandler) : IDbCommandFactory
{
    public DbCommand CreateDbCommand()
    {
        return new DbCommandMock(dbCommandMockHandler);
    }
}