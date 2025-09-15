using System.Runtime.CompilerServices;
using Terrasoft.Core;
using Terrasoft.Core.Configuration;
using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Mocking.Extensions;

public static class UserConnectionExtensions
{
    [UnsafeAccessor(UnsafeAccessorKind.Method, Name = "set_CurrentUser")]
    public static extern void set_CurrentUser(this UserConnection @this, SysUserInfo value);
    
    [UnsafeAccessor(UnsafeAccessorKind.Method, Name = "set_DBSecurityEngine")]
    public static extern void set_DBSecurityEngine(this UserConnection @this, DBSecurityEngine value);
}