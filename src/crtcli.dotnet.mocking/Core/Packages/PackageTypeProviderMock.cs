using System.Reflection;
using NSubstitute;
using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Mocking.Core.Packages;

public static class PackageTypeProviderMock
{
    public static readonly Type InterfaceType =
        Type.GetType("Terrasoft.Core.Packages.IPackageTypeProvider, Terrasoft.Core", true)!;
    
    private static readonly MethodInfo GetPackageTypeMethod =
            InterfaceType.GetMethod("GetPackageType", [typeof(Guid)])
        ?? throw new MissingMethodException(InterfaceType.FullName, "GetPackageType");
    
    
    public static object CreateNew(IPackageTypeProviderMock packageTypeProviderMock)
    {
        var packageTypeProvider = Substitute.For([InterfaceType], []);
        
        GetPackageTypeMethod
            .Invoke(packageTypeProvider, [Guid.Empty])
            .ReturnsForAnyArgs(x => packageTypeProviderMock.GetPackageType(x.Arg<Guid>()));
        
        return packageTypeProvider;
    }
}