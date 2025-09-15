using System.Collections.ObjectModel;
using System.Diagnostics.CodeAnalysis;
using Terrasoft.Core;
using Terrasoft.Core.Configuration;
using Terrasoft.Core.DcmProcess;
using Terrasoft.Core.Entities;
using Terrasoft.Core.Process;

namespace CrtCli.Dotnet.Mocking.Core.Process;

public class ProcessEngineImplMock : IProcessEngine
{
    public void Initialize(UserConnection userConnection)
    {
        UserConnection = userConnection;
    }

    public void SetProcessPropertiesData(Terrasoft.Core.Process.Process process)
    {
        throw new NotImplementedException();
    }

    public void SetProcess(Terrasoft.Core.Process.Process process)
    {
        throw new NotImplementedException();
    }

    public bool RemoveCurrentProcess(string processUId)
    {
        throw new NotImplementedException();
    }

    public bool RemoveProcessPropertiesData(string processUId)
    {
        throw new NotImplementedException();
    }

    public void RemoveSubProcessPropertiesDataByOwnerProcessUId(string ownerProcessUId)
    {
        throw new NotImplementedException();
    }

    public void ThrowSignal(ProcessExecutingContext context, string signal)
    {
        throw new NotImplementedException();
    }

    public void ThrowMessage(string processUId, string message)
    {
        throw new NotImplementedException();
    }

    public bool TryGetSysProcessData(Guid sysProcessDataId, [UnscopedRef] out SysProcessData sysProcessData)
    {
        throw new NotImplementedException();
    }

    public SysProcessData GetProcessFromDBByUId(Guid processUId)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process FindProcessByUId(string processUId, bool findInDB)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process FindProcessByUId(string processUId)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process GetProcessByUId(string processUId, bool findInDB)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process GetProcessByUId(string processUId)
    {
        throw new NotImplementedException();
    }

    public Dictionary<string, string> FindProcessPropertiesDataByUId(string processUId)
    {
        throw new NotImplementedException();
    }

    public void AddProcessListener(
        Guid recordId, 
        Guid entitySchemaUId, 
        Guid processElementUId, 
        string conditionData = null,
        string changedColumns = null, 
        EntityChangeType entityChangeType = EntityChangeType.Updated)
    {
        throw new NotImplementedException();
    }

    public void AddProcessListener(
        Entity entity, 
        Guid processElementUId, 
        string conditionData = null,
        string changedColumns = null, 
        EntityChangeType entityChangeType = EntityChangeType.Updated)
    {
        throw new NotImplementedException();
    }

    public void RemoveProcessListener(
        Guid entityId, 
        Guid entitySchemaUId, 
        Guid processElementUId,
        EntityChangeType entityChangeType = EntityChangeType.Updated, 
        Guid? workspaceId = null)
    {
        throw new NotImplementedException();
    }

    public bool RemoveActivityProcessListener(Guid activityId, Guid processElUId, Guid activityStatusId)
    {
        throw new NotImplementedException();
    }

    public bool CompleteExecuting(Guid processElementUId, params object[] parameters)
    {
        throw new NotImplementedException();
    }

    public bool ExecuteProcessElementByUId(Guid processElementUId, params object[] parameters)
    {
        throw new NotImplementedException();
    }

    public bool ExecuteProcessElement(ProcessActivity processElement, params object[] parameters)
    {
        throw new NotImplementedException();
    }

    public bool ExecuteProcessFlowElement(ProcessFlowElement processElement, params object[] parameters)
    {
        throw new NotImplementedException();
    }

    public bool TryGetSysProcessId(Guid processElementUId, [UnscopedRef] out Guid sysProcessId)
    {
        throw new NotImplementedException();
    }

    public ProcessActivity FindProcessElementByUId(Guid processElementUId)
    {
        throw new NotImplementedException();
    }

    public ProcessFlowElement FindProcessFlowElementByUId(Guid processElementUId)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process FindProcessByProcessElementUId(Guid processElementUId)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process GetProcessByElementUId(Guid elementUId)
    {
        throw new NotImplementedException();
    }

    public void LinkProcessToEntity(Terrasoft.Core.Process.Process process, Guid entitySchemaUId, Guid entityId)
    {
        throw new NotImplementedException();
    }

    public bool GetIsProcessLinkedToEntity(
        Terrasoft.Core.Process.Process process, 
        Guid entitySchemaUId, Guid entityId)
    {
        throw new NotImplementedException();
    }

    public Collection<ProcessListener> GetProcessListeners(
        UserConnection userConnection, 
        Entity entity, 
        EntityChangeType changeType)
    {
        throw new NotImplementedException();
    }

    public Collection<ProcessListener> GetProcessListeners(
        UserConnection userConnection, 
        Entity entity, Guid entityId, 
        EntityChangeType changeType,
        bool checkIsColumnChanged)
    {
        throw new NotImplementedException();
    }

    public Collection<ProcessSchemaListener> GetProcessSchemaListeners(Entity entity, EntityChangeType changeType)
    {
        throw new NotImplementedException();
    }

    public Collection<ProcessDescriptor> ContinueExecuting(
        UserConnection userConnection, 
        Entity entity, 
        Collection<ProcessListener> processListeners)
    {
        throw new NotImplementedException();
    }

    public Collection<ProcessDescriptor> RunProcesses(
        UserConnection userConnection, 
        Entity entity, 
        Collection<ProcessSchemaListener> listeners)
    {
        throw new NotImplementedException();
    }

    public void RunProcessByStartTimerEvent(Guid processSchemaId, Guid processSchemaElementUId)
    {
        throw new NotImplementedException();
    }

    public Terrasoft.Core.Process.Process RunDcmProcess(Guid entityPrimaryColumnValue, DcmSchema dcmSchema)
    {
        throw new NotImplementedException();
    }

    public void CancelNotEnabledDcmProcess(Guid entitySchemaUId, Guid entityId)
    {
        throw new NotImplementedException();
    }

    public Collection<Terrasoft.Core.Process.Process> RunDcmProcesses(
        Entity entity, 
        IList<ProcessListener> listeners)
    {
        throw new NotImplementedException();
    }

    public void SynchronizeProcessNotification(
        IEntity activity, 
        ProcessExecutingContext savingContext)
    {
        throw new NotImplementedException();
    }

    public void ActualizeProcessSchemaListeners(
        Entity entity, 
        Collection<ProcessSchemaListener> listeners)
    {
        throw new NotImplementedException();
    }

    public BaseProcessSchemaElement GetSchemaElement(Guid processElementId)
    {
        throw new NotImplementedException();
    }

    public ProcessDescriptor CompleteExecuting(
        Guid processElementId, 
        IReadOnlyDictionary<string, object> parameterValues,
        params object[] arguments)
    {
        throw new NotImplementedException();
    }

    public UserConnection UserConnection { get; set; }

    public ProcessSchemaManager ProcessSchemaManager => UserConnection.ProcessSchemaManager;
    
    public DcmSchemaManager DcmSchemaManager => UserConnection.DcmSchemaManager;

    public EntitySchemaManager EntitySchemaManager => UserConnection.EntitySchemaManager;

    public int MaxLoopCount => 100;
    
    public Dictionary<ProcessStatus, Guid> ProcessActivityStatus => throw new NotImplementedException();
    
    public Dictionary<Guid, List<ProcessSchemaListener>> SysEntityPrcStartEvents => 
        throw new NotImplementedException();
    
    public IProcessExecutor ProcessExecutor => throw new NotImplementedException();
    
    public string ProcessListenersColumnName => "ProcessListeners";
}