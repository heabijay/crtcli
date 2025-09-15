using CrtCli.Dotnet.Context.Exceptions;

namespace CrtCli.Dotnet.Context;

public class PackageContextAccessor : IPackageContextAccessor
{
    private IPackageContext? _packageContext;

    public IPackageContext PackageContext => GetPackageContext();

    public IPackageContext GetPackageContext()
    {
        if (_packageContext is null)
        {
            throw new PackageContextNotRegisteredInAccessorException();
        }

        return _packageContext;
    }
    
    public void SetPackageContext(IPackageContext packageContext)
    {
        _packageContext = packageContext;
    }
}