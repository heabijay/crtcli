using System;
using System.Runtime.Serialization;
using Newtonsoft.Json;

namespace CrtCli.Packages.Tunneling.Contracts;

[DataContract]
public class ErrorResponse
{
    [DataMember(Name = "type")] [JsonProperty("type")] public string Type { get; set; }
    
    [DataMember(Name = "message")] [JsonProperty("message")] public string Message { get; set; }
    
    [DataMember(Name = "full_message")] [JsonProperty("full_message")] public string FullMessage { get; set; }

    
    public static ErrorResponse FromException(Exception exception)
    {
        return new ErrorResponse()
        {
            Type = exception.GetType().FullName,
            Message = exception.Message,
            FullMessage = exception.ToString(),
        };
    }
}