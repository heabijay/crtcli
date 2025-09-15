namespace CrtCli.Dotnet.Context.Exceptions;

internal class PackageContextNotRegisteredInAccessorException() 
    : Exception("Current Package Context is not set, please check your DI configuration and usage of IPackageContextAccessor service")
{
}