using System.Data;
using System.Data.Common;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public interface IDbCommandMockHandler
{
    DbDataReader Handle(IDbCommand dbCommand, DbCommandQueryType queryType);
}