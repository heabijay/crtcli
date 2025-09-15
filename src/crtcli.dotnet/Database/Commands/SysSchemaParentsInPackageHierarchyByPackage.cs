using System.Data;
using System.Data.Common;
using CrtCli.Dotnet.Context;
using CrtCli.Dotnet.Database.Commands.Abstraction;
using CrtCli.Dotnet.Mocking.Core.DB;
using CrtCli.Dotnet.Utilities;
using Microsoft.Extensions.Logging;

namespace CrtCli.Dotnet.Database.Commands;

internal static class SysSchemaParentsInPackageHierarchyByPackage
{
    public record Request(Guid StartSchemaUId) : IDbCommandMockRequest;

    public class Parser : IDbCommandMockRequestParser<Request>
    {
        public bool TryParse(IDbCommand dbCommand, out Request request)
        {
            const string expectedCommandText = 
                """
                SELECT * 
                FROM public."tsp_GetSysSchemaParentsInPackageHierarchyByPackage"(
                    @StartSchemaUId, 
                    @WorkspaceId)
                """;

            if (!DbCommandTextEqualityComparer.Instance.Equals(dbCommand.CommandText, expectedCommandText))
            {
                request = null!;
                return false;
            }

            var startSchemaUIdParameter = (DbParameterMock)dbCommand.Parameters["StartSchemaUId"];
            var startSchemaUId = (Guid)startSchemaUIdParameter.Value!;

            request = new Request(startSchemaUId);
            return true;
        }
    }

    public class Handler(ILogger<Handler> logger, IPackageContext packageContext) : IDbCommandMockRequestHandler<Request>
    {
        public DbDataReader Handle(Request request)
        {
            logger.LogDebug("Returning only single schemaUId {SchemaUId} instead of full hierarchy in package", request.StartSchemaUId);

            var schemaContext = packageContext
                .Schemas
                .FirstOrDefault(x => x.Descriptor.UId == request.StartSchemaUId);

            if (schemaContext is null)
            {
                throw new SchemaContextWithUIdNotFound(request.StartSchemaUId);
            }

            return new DataTableReader(new DataTable()
            {
                Columns =
                {
                    { "Id", typeof(Guid) },
                    { "UId", typeof(Guid) },
                    { "Name", typeof(string) },
                    { "MetaData", typeof(byte[]) },
                    { "ParentId", typeof(Guid) },
                    { "ModifiedOn", typeof(DateTime) },
                    { "PackageLevel", typeof(int) },
                    { "SchemaLevel", typeof(int) },
                },
                Rows =
                {
                    { 
                        schemaContext.Descriptor.UId, 
                        schemaContext.Descriptor.UId, 
                        schemaContext.Descriptor.Name, 
                        schemaContext.GetMetadata(), 
                        Guid.Empty, 
                        DateTime.MinValue, 
                        0, 
                        0 
                    }
                }
            });
        }
    }

    public class SchemaContextWithUIdNotFound(Guid schemaUId)
        : Exception($"Schema with UId '{schemaUId}''was not found in current context")
    {
        public Guid SchemaUId { get; } = schemaUId;
    }
}