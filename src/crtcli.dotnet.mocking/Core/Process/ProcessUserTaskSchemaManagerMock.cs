using Microsoft.Extensions.Logging;
using Terrasoft.Core.Process;

namespace CrtCli.Dotnet.Mocking.Core.Process;

public class ProcessUserTaskSchemaManagerMock : ProcessUserTaskSchemaManager
{
    private readonly ILogger<ProcessUserTaskSchemaManagerMock> _logger = MockingLogging.CreateLogger<ProcessUserTaskSchemaManagerMock>();
    
    public ProcessUserTaskSchemaManagerMock()
    {
        Name = "ProcessUserTaskSchemaManager";
    }
    
    protected override bool InitializeItems()
    {
        _logger.LogDebug("`ProcessUserTaskSchemaManagerMock.InitializeItems` was called, but no action was taken");
        
        return true;
    }
}