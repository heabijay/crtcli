using System.Runtime.CompilerServices;
using Terrasoft.Core;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class AppConnectionExtensions
{
    [UnsafeAccessor(UnsafeAccessorKind.Method, Name = "set_AppManagerProvider")]
    public static extern void set_AppManagerProvider(this AppConnection @this, ManagerProvider value);
}