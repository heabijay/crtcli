#if NETFRAMEWORK

using System;
using System.Threading.Tasks;
using System.Web;
using System.Web.Routing;
using CrtCli.Packages.Tunneling.Commands;
using Newtonsoft.Json;
using Terrasoft.Common;
using Terrasoft.Core;
using Terrasoft.Core.Factories;

namespace CrtCli.Packages.Tunneling;

public sealed partial class TunnelingService
{
    public Task Connect(string host, ushort port)
    {
        // Check ConnectRouteHandler & ConnectHttpHandler for actual implementation
        return Task.CompletedTask;
    }
    

    internal sealed class ConnectRouteHandler : IRouteHandler
    {
        public IHttpHandler GetHttpHandler(RequestContext requestContext)
        {
            return new ConnectHttpHandler();
        }
    }

    private sealed class ConnectHttpHandler : IHttpHandler
    {
        public bool IsReusable => true;
        
        public void ProcessRequest(HttpContext context)
        {
            try
            {
                var userConnection = GetSystemUserConnection();
                
                CheckCanUseTunneling(userConnection, context.User?.Identity?.Name);
                
                var host = context.Request.QueryString["host"];
                var portString = context.Request.QueryString["port"];

                if (string.IsNullOrEmpty(host))
                {
                    throw new ArgumentNullOrEmptyException("host");
                }
                
                if (string.IsNullOrEmpty(portString))
                {
                    throw new ArgumentNullOrEmptyException("port");
                }
                
                if (!ushort.TryParse(portString, out var port))
                {
                    throw new ArgumentException($"Cannot parse port value '{portString}' to ushort", nameof(port));
                }
                
                context.AcceptWebSocketRequest(webSocket => 
                    ConnectInternalWhenWebSocketReadyAsync(webSocket.WebSocket, host, port));
            }
            catch (Exception e)
            {
                context.Response.ContentType = "application/json";
                context.Response.Write(JsonConvert.SerializeObject(new
                {
                    error = ErrorResponse.FromException(e)
                }));
            }
            
            context.Response.End();
        }

        private static UserConnection GetSystemUserConnection()
        {
            return ClassFactory.Get<AppConnection>().SystemUserConnection;
        }
    }
}

#endif