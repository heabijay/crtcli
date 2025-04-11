using System;
using System.Configuration;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Net.WebSockets;
using System.ServiceModel;
using System.ServiceModel.Activation;
using System.ServiceModel.Web;
using System.Threading;
using System.Threading.Tasks;
using System.Web.SessionState;
using CrtCli.Packages.Tunneling.Contracts;
using CrtCli.Packages.Tunneling.Utilities;
using Newtonsoft.Json;
using Terrasoft.Core;
using Terrasoft.Core.Configuration;
using Terrasoft.Core.Factories;
using Terrasoft.Web.Common;

namespace CrtCli.Packages.Tunneling;

[DefaultBinding(typeof(ITunnelingService))]
[AspNetCompatibilityRequirements(RequirementsMode = AspNetCompatibilityRequirementsMode.Required)]
public sealed partial class TunnelingService : BaseService, ITunnelingService, IReadOnlySessionState
{
    private const int DefaultStreamCopyBufferSize = 81920;
    private const string CrtCliTunnelingVersionHeaderKey = "X-CrtCli-Tunneling-Version";
    private static readonly string CrtCliTunnelingVersionHeaderValue = typeof(ITunnelingService).Assembly.GetName().Version.ToString();

    public void GetInformation()
    {
        var httpContext = HttpContextAccessor.GetInstance();
        var result = GetInformationInternal();

        httpContext.Response.ContentType = "application/json";
        httpContext.Response.Headers[CrtCliTunnelingVersionHeaderKey] = CrtCliTunnelingVersionHeaderValue;
        httpContext.Response.Write(JsonConvert.SerializeObject(result));
    }

    private TunnelingInformationResponse GetInformationInternal()
    {
        try
        {
            CheckCanUseTunneling(UserConnection, UserConnection.CurrentUser.Id);

            return new TunnelingInformationResponse()
            {
                Version = CrtCliTunnelingVersionHeaderValue,
                Allowed = true,
            };
        }
        catch (Exception ex)
        {
            return new TunnelingInformationResponse()
            {
                Version = CrtCliTunnelingVersionHeaderValue,
                Allowed = false,
                Error = ErrorResponse.FromException(ex),
            };
        }
    }
    
    private static async Task ConnectInternalWhenWebSocketReadyAsync(WebSocket webSocket, string host, ushort port)
    {
        using var tcpClient = new TcpClient();
        
        tcpClient.NoDelay = true;
        
        await tcpClient.ConnectAsync(host, port).ConfigureAwait(false);
        
        using var tcpClientStream = tcpClient.GetStream();
        using var webSocketBinaryStream = new WebSocketBinaryStreamAdapter(webSocket);
        
        var cts = new CancellationTokenSource();

        try
        {
            await Task.WhenAny(
                    webSocketBinaryStream.CopyToAsync(tcpClientStream, DefaultStreamCopyBufferSize, cts.Token),
                    tcpClientStream.CopyToAsync(webSocketBinaryStream, DefaultStreamCopyBufferSize, cts.Token))
                .ConfigureAwait(false);
        }
        finally
        {
            cts.Cancel();
        }
    }

    public void GetConnectionStrings()
    {
        try
        {
            CheckCanUseTunneling(UserConnection, UserConnection.CurrentUser.Id);

            var connectionStringsCollection = UserConnection
                .AppConnection
                .AppSettings
                .RootConfiguration
                .ConnectionStrings
                .ConnectionStrings;
            
            var connectionStrings = connectionStringsCollection
                .Cast<ConnectionStringSettings>()
                .ToDictionary(x => x.Name, x => x.ConnectionString);

            var httpContext = HttpContextAccessor.GetInstance();

            httpContext.Response.ContentType = "application/json";
            httpContext.Response.Write(JsonConvert.SerializeObject(connectionStrings));
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

    private static void CheckCanUseTunneling(UserConnection userConnection, Guid sysAdminUnitId)
    {
        userConnection.DBSecurityEngine.CheckCanExecuteOperation("CanManageSolution", sysAdminUnitId);
    }

    private static void CheckCanUseTunneling(UserConnection userConnection, string userName)
    {
        var sysAdminUnit = new SysAdminUnit(userConnection);
        
        sysAdminUnit.FetchFromDB("Name", userName, false);
        
        CheckCanUseTunneling(userConnection, sysAdminUnit.Id);
    }
}