using System;
using Common.Logging;
using Terrasoft.Web.Common;

namespace CrtCli.Packages.Tunneling;

internal partial class TunnelingAppEventListener : AppEventListenerBase
{
    public override void OnAppStart(AppEventContext context)
    {
        try
        {
#if NETFRAMEWORK
            OnAppStart_NetFramework(context);
#endif
        }
        catch (Exception e)
        {
            LogManager.GetLogger<TunnelingAppEventListener>().Error("Cannot initialize crtcli.tunneling on Creatio App Start", e);
        }
    }
}