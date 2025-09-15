using Microsoft.Extensions.Logging;
using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Mocking.Core.Packages;

public class AssemblyPackageTypeProviderMock : IPackageTypeProviderMock
{
    private readonly ILogger<AssemblyPackageTypeProviderMock> _logger = MockingLogging.CreateLogger<AssemblyPackageTypeProviderMock>();
    
    public PackageType GetPackageType(Guid packageUId)
    {
        _logger.LogTrace("`IPackageTypeProviderMock.GetPackageType` for packageUId '{PackageUId}' returned `PackageType.Assembly`", packageUId);
        
        return PackageType.Assembly;
    }
}