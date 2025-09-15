using Microsoft.Extensions.Logging;
using Terrasoft.Core.Entities;

namespace CrtCli.Dotnet.Mocking.Core.Entities;

public class EntitySchemaManagerMock : EntitySchemaManager
{
    private readonly ILogger<EntitySchemaManagerMock> _logger = MockingLogging.CreateLogger<EntitySchemaManagerMock>();
    
    public EntitySchemaManagerMock()
    {
        Name = "EntitySchemaManager";
    }
    
    protected override bool InitializeItems()
    {
        _logger.LogDebug("`EntitySchemaManagerMock.InitializeItems` was called, but no action was taken");
        
        return true;
    }
}