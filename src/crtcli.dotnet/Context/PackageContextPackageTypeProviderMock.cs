using CrtCli.Dotnet.Context.Exceptions;
using CrtCli.Dotnet.Mocking.Core.Packages;
using Microsoft.Extensions.Logging;
using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Context;

internal class PackageContextPackageTypeProviderMock(
    ILogger<PackageContextPackageTypeProviderMock> logger,
    IPackageContext packageContext)
    : IPackageTypeProviderMock
{
    public PackageType GetPackageType(Guid packageUId)
    {
        if (packageContext.Descriptor.UId == packageUId)
        {
            logger.LogTrace(
                "`IPackageTypeProviderMock.GetPackageType` for packageUId '{PackageUId}' returned `PackageType.{PackageType}`", 
                packageUId,
                packageContext.Descriptor.Type);

            return packageContext.Descriptor.Type;
        }
        
        throw new CannotFindPackageInContextException(packageUId);
    }
}