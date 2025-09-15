using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Mocking.Core.Packages;

public interface IPackageTypeProviderMock
{
    PackageType GetPackageType(Guid packageUId);
}