namespace CrtCli.Dotnet.Context.Exceptions;

[Serializable]
internal class CannotFindPackageInContextException(Guid uId)
    : Exception($"Cannot find Package with UId '{uId}' in current Context")
{
    public Guid UId { get; } = uId;
}