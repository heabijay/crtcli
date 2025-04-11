using System.Runtime.Serialization;
using Newtonsoft.Json;

namespace CrtCli.Packages.Tunneling.Contracts;

[DataContract]
public class TunnelingInformationResponse
{
    [DataMember(Name = "version")] [JsonProperty("version")] public string Version { get; set; }

    [DataMember(Name = "allowed")] [JsonProperty("allowed")] public bool Allowed { get; set; }
    
    [DataMember(Name = "error")] [JsonProperty("error")] public ErrorResponse Error { get; set; }
}