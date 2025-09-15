using System.Runtime.CompilerServices;
using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class DBEngineExtensions
{
    [UnsafeAccessor(UnsafeAccessorKind.Method, Name = "set_CurrentSchemaName")]
    public static extern void set_CurrentSchemaName(this DBEngine @this, string value);
}