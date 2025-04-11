#if NETFRAMEWORK

using System.Web.Routing;
using Terrasoft.Web.Common;

namespace CrtCli.Packages.Tunneling;

internal partial class TunnelingAppEventListener
{
    private void OnAppStart_NetFramework(AppEventContext context)
    {
        RouteTable.Routes.Insert(
            0, 
            new Route("rest/crtcli/tunneling/connect", new TunnelingService.ConnectRouteHandler()));
    }
}

#endif