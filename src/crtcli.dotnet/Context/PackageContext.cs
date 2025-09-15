using Newtonsoft.Json;

namespace CrtCli.Dotnet.Context;

internal class PackageContext(string basePath) : IPackageContext
{
    private PackageDescriptorExtended? _descriptor;
    private IEnumerable<IPackageSchemaContext>? _schemas;

    public PackageDescriptorExtended Descriptor => _descriptor ??= ReadDescriptor();

    public IEnumerable<IPackageSchemaContext> Schemas => _schemas ??= ReadSchemas();


    private PackageDescriptorExtended ReadDescriptor()
    {
        var descriptorJson = File.ReadAllText(Path.Combine(basePath, "descriptor.json"));
        
        return JsonConvert.DeserializeObject<DescriptorWrapper<PackageDescriptorExtended>>(descriptorJson)?.Descriptor
               ?? throw new JsonException("$.Descriptor property was not found in package descriptor");
    }
    
    private IEnumerable<IPackageSchemaContext> ReadSchemas()
    {
        var schemasFolder = Path.Combine(basePath, "Schemas");

        if (!Directory.Exists(schemasFolder))
        {
            return [];
        }
        
        return Directory
            .EnumerateDirectories(schemasFolder)
            .Select(dir => new PackageSchemaContext(dir))
            .ToList();
    }
}