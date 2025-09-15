using Terrasoft.Core.DB;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DBTypeConverterMock : DBTypeConverter
{
    public override object ValueToDBValue(object value) => throw new NotImplementedException();

    public override Guid DBValueToGuid(object value) => (Guid)Convert.ChangeType(value, typeof(Guid));
}