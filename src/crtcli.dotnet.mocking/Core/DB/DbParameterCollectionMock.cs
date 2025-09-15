using System.Collections;
using System.Data.Common;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DbParameterCollectionMock : DbParameterCollection
{
    private readonly List<DbParameterMock> _internalCollection = new(5);
    
    
    public override int Add(object value)
    {
        _internalCollection.Add((DbParameterMock)value);
        
        return _internalCollection.Count - 1;
    }

    public override void Clear() => _internalCollection.Clear();

    public override bool Contains(object value) => _internalCollection.Contains(value);

    public override int IndexOf(object value) => _internalCollection.IndexOf((DbParameterMock)value);

    public override void Insert(int index, object value) => _internalCollection.Insert(index, (DbParameterMock)value);

    public override void Remove(object value) => _internalCollection.Remove((DbParameterMock)value);

    public override void RemoveAt(int index) => _internalCollection.RemoveAt(index);

    public override void RemoveAt(string parameterName)
    {
        throw new NotImplementedException();
    }

    protected override void SetParameter(int index, DbParameter value)
    {
        throw new NotImplementedException();
    }

    protected override void SetParameter(string parameterName, DbParameter value)
    {
        throw new NotImplementedException();
    }

    public override int Count => _internalCollection.Count;
    
    public override object SyncRoot => _internalCollection;

    public override int IndexOf(string parameterName)
    {
        throw new NotImplementedException();
    }

    public override bool Contains(string value)
    {
        throw new NotImplementedException();
    }

    public override void CopyTo(Array array, int index)
    {
        throw new NotImplementedException();
    }

    public override IEnumerator GetEnumerator() => _internalCollection.GetEnumerator();

    protected override DbParameter GetParameter(int index)
    {
        return _internalCollection[index];
    }

    protected override DbParameter GetParameter(string parameterName)
    {
        return _internalCollection.Single(x => x.ParameterName == parameterName);
    }

    public override void AddRange(Array values) => _internalCollection.AddRange(values.Cast<DbParameterMock>());
}