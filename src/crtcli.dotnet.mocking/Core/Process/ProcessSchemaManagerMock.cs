using Microsoft.Extensions.Logging;
using Terrasoft.Core;
using Terrasoft.Core.Process;

namespace CrtCli.Dotnet.Mocking.Core.Process;

public class ProcessSchemaManagerMock : ProcessSchemaManager
{
    private readonly ILogger<ProcessSchemaManagerMock> _logger = MockingLogging.CreateLogger<ProcessSchemaManagerMock>();

    public ProcessSchemaManagerMock()
    {
        Name = "ProcessSchemaManager";
    }

    protected override ProcessSchema InitializeDefSchema()
    {
        return new ProcessSchemaMock(this);
    }

    protected override bool InitializeItems(Guid itemUId)
    {
        _logger.LogDebug("`ProcessSchemaManagerMock.InitializeItems` was called for ItemUId '{ItemUId}', but no action was taken", itemUId);
        
        return true;
    }

    protected override void ActualizeSchemaInfo(ProcessSchema schema, Guid schemaId, UserConnection connection)
    {
        _logger.LogDebug("`ProcessSchemaManagerMock.ActualizeSchemaInfo` was called for SchemaUId '{SchemaUId}', but no action was taken", schema.UId);
    }
}