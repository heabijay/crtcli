using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Context;

public interface IPackageSchemaContext
{
    SchemaDescriptor Descriptor { get; }

    byte[] GetMetadata();
}