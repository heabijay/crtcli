using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Context;

public record DescriptorWrapper<TDescriptor>(TDescriptor Descriptor) where TDescriptor: Descriptor;