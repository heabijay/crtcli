namespace CrtCli.Dotnet.Context;

public interface IPackageContext
{
    PackageDescriptorExtended Descriptor { get; }
    
    IEnumerable<IPackageSchemaContext> Schemas { get; }
}