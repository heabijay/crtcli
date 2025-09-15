using System.Runtime.CompilerServices;
using Terrasoft.Core.Packages;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class PackageExtensions
{
    [UnsafeAccessor(UnsafeAccessorKind.Method, Name = "get_PackageType")]
    public static extern PackageType get_PackageType(this Package @this);
}