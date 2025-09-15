using System.Runtime.CompilerServices;
using Terrasoft.Core.Entities;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class EntityExtensions
{
    [UnsafeAccessor(UnsafeAccessorKind.Method, Name = "get_ColumnValues")]
    public static extern EntityColumnValueCollection get_ColumnValues(this Entity @this);
}