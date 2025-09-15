using Newtonsoft.Json;
using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Context;

internal class PackageSchemaContext(string basePath) : IPackageSchemaContext
{
    private SchemaDescriptor? _descriptor;

    public SchemaDescriptor Descriptor => _descriptor ??= ReadDescriptor();

    private SchemaDescriptor ReadDescriptor()
    {
        var descriptorJson = File.ReadAllText(Path.Combine(basePath, "descriptor.json"));
        
        return JsonConvert.DeserializeObject<DescriptorWrapper<SchemaDescriptor>>(descriptorJson)?.Descriptor
            ?? throw new JsonException("$.Descriptor property was not found in schema descriptor");
    }
    
    public byte[] GetMetadata()
    {
        return File.ReadAllBytes(Path.Combine(basePath, "metadata.json"));
    }
}