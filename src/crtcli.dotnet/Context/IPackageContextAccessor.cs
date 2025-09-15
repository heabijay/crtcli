namespace CrtCli.Dotnet.Context;

public interface IPackageContextAccessor
{
    IPackageContext PackageContext { get; }
}