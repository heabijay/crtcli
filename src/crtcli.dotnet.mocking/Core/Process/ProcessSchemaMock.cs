using CrtCli.Dotnet.Mocking;
using Microsoft.Extensions.Logging;

// ReSharper disable once CheckNamespace
namespace Terrasoft.Core.Process;

public class ProcessSchemaMock : ProcessSchema
{
    private readonly ILogger<ProcessSchemaMock> _logger = MockingLogging.CreateLogger<ProcessSchemaMock>();

    public ProcessSchemaMock(ISchemaManager processSchemaManager) : base(processSchemaManager)
    {
    }

    public ProcessSchemaMock(ISchemaManager processSchemaManager, ProcessBasedSchema ownerSchema) : base(processSchemaManager, ownerSchema)
    {
    }

    public ProcessSchemaMock(ProcessSchema source) : base(source)
    {
    }

    protected override void LoadResources()
    {
        _logger.LogDebug("`ProcessSchemaMock[:{Name}].LoadResources` was called, but no action was taken", Name);
    }

    public override object Clone()
    {
        return new ProcessSchemaMock(this);
    }

    public override void SynchronizeParameters()
    {
        _logger.LogDebug("`ProcessSchemaMock[:{Name}].SynchronizeParameters` was called, but no action was taken", Name);
    }
}