using System.ServiceModel;
using System.ServiceModel.Web;
using System.Threading.Tasks;

namespace CrtCli.Packages.Tunneling;

[ServiceContract(Name = "crtcli/tunneling")]
public interface ITunnelingService
{
    [OperationContract]
    [WebGet(
        UriTemplate = "/",
        BodyStyle = WebMessageBodyStyle.Bare,
        RequestFormat = WebMessageFormat.Json,
        ResponseFormat = WebMessageFormat.Json)]
    void GetInformation();

    [OperationContract]
    [WebGet(
        UriTemplate = "connect?host={host}&port={port}", 
        BodyStyle = WebMessageBodyStyle.Bare,
        RequestFormat = WebMessageFormat.Json,
        ResponseFormat = WebMessageFormat.Json)]
    Task Connect(string host, ushort port);

    [OperationContract]
    [WebGet(
        UriTemplate = "connection-strings",
        BodyStyle = WebMessageBodyStyle.Bare,
        RequestFormat = WebMessageFormat.Json,
        ResponseFormat = WebMessageFormat.Json)]
    void GetConnectionStrings();
}