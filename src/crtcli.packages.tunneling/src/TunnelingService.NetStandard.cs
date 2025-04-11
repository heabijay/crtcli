#if NETSTANDARD

using System;
using System.Net;
using System.ServiceModel;
using System.ServiceModel.Web;
using System.Threading.Tasks;
using CrtCli.Packages.Tunneling.Contracts;
using CrtCli.Packages.Tunneling.Utilities;

namespace CrtCli.Packages.Tunneling;

public sealed partial class TunnelingService
{
    public async Task Connect(string hostname, ushort port)
    {
        try
        {
            CheckCanUseTunneling(UserConnection, UserConnection.CurrentUser.Id);
            
            var httpContext = HttpContextAccessor.GetInstance().GetInnerHttpContext();
            
            var websocketMiddleware = WebSocketMiddleware.CreateInstance(
                httpContext.RequestServices,
                async context =>
                {
                    var webSocket = await context.WebSockets.AcceptWebSocketAsync().ConfigureAwait(false);
                    
                    await ConnectInternalWhenWebSocketReadyAsync(webSocket, hostname, port).ConfigureAwait(false);
                });
            
            await WebSocketMiddleware.Invoke(websocketMiddleware, httpContext).ConfigureAwait(false);
        }
        catch (FaultException)
        {
            throw;
        }
        catch (Exception ex)
        {
            throw new WebFaultException<ErrorResponse>(ErrorResponse.FromException(ex), HttpStatusCode.BadRequest);
        }
    }
}

#endif
