using System.Data;
using System.Data.Common;
using System.Text;
using CrtCli.Dotnet.Mocking.Extensions;
using Terrasoft.Common;
using Terrasoft.Core;
using Terrasoft.Core.Configuration;
using Terrasoft.Core.DB;
using Terrasoft.Core.DB.DbStructureReader;
using Terrasoft.Core.Entities;

namespace CrtCli.Dotnet.Mocking.Core.DB;

public class DBEngineMock : DBEngine
{
    private const string BinaryDataValueTypeName = "BYTEA";

    public DBEngineMock()
    {
        this._startDelimiter = "\"";
        this._endDelimiter = "\"";
        this._topStatement = "LIMIT";
        this._parameterMarker = "@";
        this._anyStringPatternSymbol = "%";
        this.set_CurrentSchemaName("public");
    }

    public override DBEngineType DBEngineType => DBEngineType.PostgreSql;

    [Obsolete("7.13.0 | Use NeedWrapFilterTextInUpperFunction(EntitySchemaQueryFilter filter) instead")] 
    public override bool NeedWrapInUpperFunction => false;
    
    [Obsolete("7.13.2 | Use NeedCastOrderedTextColumn(TextDataValueType orderedColumnDataValueType) instead")] 
    public override bool CanOrderByMaxSizeTextColumn => false;

    public override bool AllowsMars => true;
    
    
    protected override string GetStartCheckIntegrityBlock(string checkEntitySchemaName)
    {
        throw new NotImplementedException();
    }

    protected override void BuildDateTimeValueSqlText(StringBuilder sb, DateTime dateTimeValue)
    {
        throw new NotImplementedException();
    }

    protected override void BuildDateValueSqlText(StringBuilder sb, DateTime dateTimeValue)
    {
        throw new NotImplementedException();
    }

    protected override void BuildTimeValueSqlText(StringBuilder sb, DateTime dateTimeValue)
    {
        throw new NotImplementedException();
    }

    protected override void BuildTimeValueSqlText(StringBuilder sb, TimeSpan timeValue)
    {
        throw new NotImplementedException();
    }

    protected override void BuildFloatValueSqlText(StringBuilder sb, double doubleValue)
    {
        throw new NotImplementedException();
    }

    protected override void BuildBooleanValueSqlText(StringBuilder sb, bool value)
    {
        throw new NotImplementedException();
    }

    protected override string BuildTemporaryTableSourceExpressionSqlText(string tempTableName)
    {
        throw new NotImplementedException();
    }

    protected override void BuildHierarchicalSelectSqlText(
        StringBuilder sb, 
        Select select, 
        HierarchicalSelectOptions options)
    {
        sb.Append(
            """
            SELECT
                "Id",
                "Name",
                "ParentId"
            FROM
                "$HierarchicalSelect"
            """);
    }

    protected override void BuildFirstPageWithHierarchySelectSqlText(StringBuilder sb, Select mainSelect, Select select,
        PageableSelectOptions pageableOptions, HierarchicalSelectOptions hierarchicalOptions)
    {
        throw new NotImplementedException();
    }

    protected override QueryColumnExpression CreateCastToShortTextExpression(string columnAlias)
    {
        throw new NotImplementedException();
    }

    public override bool NeedCastOrderedTextColumn(DataValueType orderedColumnDataValueType)
    {
        throw new NotImplementedException();
    }

    public override string GetBitwiseOperationSqlText(string columnName, BitwiseOperation operation, string constant)
    {
        throw new NotImplementedException();
    }

    public override bool GetIsOnlyLikeComparison(DataValueType dataValueType)
    {
        if (dataValueType is TextDataValueType textDataValueType)
        {
            return textDataValueType.IsMaxSize;
        }
        
        return false;
    }

    public override void BuildSysSettingsIntegrityCheckModifyTriggerBody(StringBuilder sb, EntitySchema referenceSchema)
    {
        throw new NotImplementedException();
    }

    public override void BuildNoLockHintSqlText(StringBuilder sb, NoLockHint noLockSqlHint)
    {
        throw new NotImplementedException();
    }

    public override void BuildIndexHintSqlText(StringBuilder sb, IndexHint indexSqlHint)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, IsNullQueryFunction isNullQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, DatePartQueryFunction datePartQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, DateAddQueryFunction dateAddQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, DateDiffQueryFunction dateDiffQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, CreateGuidQueryFunction createGuidQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, CurrentDateTimeQueryFunction currentDateTimeQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, DataLengthQueryFunction dataLengthQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQueryFunctionSqlText(StringBuilder sb, TrimQueryFunction trimQueryFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildQuerySqlText(StringBuilder sb, StoredProcedure storedProcedure)
    {
        throw new NotImplementedException();
    }

    public override void BuildNextValueSequenceQuerySqlText(StringBuilder sb, Sequence sequence)
    {
        throw new NotImplementedException();
    }

    public override void BuildCurrentValueSequenceQuerySqlText(StringBuilder sb, Sequence sequence)
    {
        throw new NotImplementedException();
    }

    public override void BuildReCreateSequenceQuerySqlText(StringBuilder sb, Sequence sequence)
    {
        throw new NotImplementedException();
    }

    public override void BuildExistSequenceQuerySqlText(StringBuilder sb, Sequence sequence)
    {
        throw new NotImplementedException();
    }

    public override void BuildDropSequenceQuerySqlText(StringBuilder sb, Sequence sequence)
    {
        throw new NotImplementedException();
    }

    public override void BuildQuerySqlText(StringBuilder sb, UserDefinedFunction userDefinedFunction)
    {
        throw new NotImplementedException();
    }

    public override void BuildIntegrityCheckDeleteTriggerHeaderBody(StringBuilder sb)
    {
        throw new NotImplementedException();
    }

    public override void BuildIntegrityCheckDeleteTriggerFooterBody(StringBuilder sb)
    {
        throw new NotImplementedException();
    }

    public override void BuildIntegrityCheckModifyTriggerHeaderBody(StringBuilder sb)
    {
        throw new NotImplementedException();
    }

    public override void BuildIntegrityCheckModifyTriggerModifyBodyCheckForNullColumn(
        StringBuilder sb,
        string columnValueName,
        EntitySchemaColumnRequirementType columnRequirementType)
    {
        throw new NotImplementedException();
    }

    public override void BuildIntegrityCheckModifyTriggerCheckModifyBody(
        StringBuilder sb, 
        string referenceSchemaName,
        string columnValueName, 
        string referenceSchemaPrimaryColumnName)
    {
        throw new NotImplementedException();
    }

    public override void BuildIntegrityCheckModifyTriggerFooterBody(StringBuilder sb, string entitySchemaName)
    {
        throw new NotImplementedException();
    }

    public override void BuildEditIntegrityCheckModifyTriggerCheckModifyBody(
        StringBuilder sb, 
        string oldEntitySchemaName,
        string entitySchemaName)
    {
        throw new NotImplementedException();
    }

    public override void BuildSysSettingsIntegrityCheckModifyTriggerHeaderBody(StringBuilder sb)
    {
        throw new NotImplementedException();
    }

    public override void BuildSysSettingsIntegrityCheckModifyTriggerBody(
        StringBuilder sb, 
        Guid id,
        SysSettingsReferenceSchemaCollection referenceSchemaList)
    {
        throw new NotImplementedException();
    }

    public override void BuildSysSettingsIntegrityCheckDeleteTriggerBody(
        StringBuilder sb, 
        string referenceSchemaName,
        string referenceSchemaPrimaryColumnName, 
        Guid sysSettingsId)
    {
        throw new NotImplementedException();
    }

    public override string GetImageLookupAfterInsertUpdateTriggerBody(string columnName)
    {
        throw new NotImplementedException();
    }

    public override string GetImageLookupAfterDeleteTriggerBody(string columnName)
    {
        throw new NotImplementedException();
    }

    public override bool TriggerBodyHasIntegrityCheckBlocks(StringBuilder sb)
    {
        throw new NotImplementedException();
    }

    public override bool GetIsBinaryDataType(string dataTypeName)
    {
        return BinaryDataValueTypeName.Equals(dataTypeName, StringComparison.OrdinalIgnoreCase);
    }

    public override string GetIndexQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetColumnIndexesQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override void BuildNullableColumnDefinitionSqlText(
        StringBuilder sb, 
        EntitySchema entitySchema, 
        string entitySchemaName,
        EntitySchemaColumn entitySchemaColumn, 
        string columnName)
    {
        throw new NotImplementedException();
    }

    public override void BuildCreateWeakNullableSqlText(
        StringBuilder sb, 
        EntitySchema entitySchema, 
        EntitySchemaColumn entitySchemaColumn,
        string columnName)
    {
        throw new NotImplementedException();
    }

    public override void BuildCreateDefValueSqlText(
        StringBuilder sb, 
        EntitySchema entitySchema, 
        EntitySchemaColumn entitySchemaColumn)
    {
        throw new NotImplementedException();
    }

    public override string GetEditPrimaryColumnDefinitionSqlText(
        IDataReader dataReader, 
        EntitySchema entitySchema,
        EntitySchemaColumn entitySchemaColumn)
    {
        throw new NotImplementedException();
    }

    public override string GetPrimaryKeyQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetForeignKeysQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetForeignKeyQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetTriggerQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetColumnForeignKeyQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetPrimaryKeyParameterValue()
    {
        throw new NotImplementedException();
    }

    public override string GetForeignKeyParameterValue()
    {
        throw new NotImplementedException();
    }

    public override string GetIsTextColumnContainGuidQuerySqlText(string entitySchemaName, string columnName)
    {
        throw new NotImplementedException();
    }

    public override string GetDanglingReferenceQuerySqlText(
        string entitySchemaName, 
        string columnName, 
        string referenceSchemaName,
        string referenceSchemaColumnName, 
        bool skipGuidExpression)
    {
        throw new NotImplementedException();
    }

    public override string GetColumnDefValue(EntitySchemaColumn entitySchemaColumn)
    {
        throw new NotImplementedException();
    }

    public override string GetColumnDefValue(EntitySchemaColumn entitySchemaColumn, bool isWrapped)
    {
        throw new NotImplementedException();
    }

    public override string GetColumnDefValue(EntitySchemaColumn entitySchemaColumn, string dbDefValue)
    {
        throw new NotImplementedException();
    }

    public override string GetUpdateSysSolutionRightAfterUnitDeleteSqlText()
    {
        throw new NotImplementedException();
    }

    public override string FormatException(DbException exception)
    {
        throw new NotImplementedException();
    }

    public override string GetSysSettingsIntegrityCheckBlockName(Guid sysSettingsId)
    {
        throw new NotImplementedException();
    }

    public override string GetSystemValueSqlText(SystemValue systemValue)
    {
        throw new NotImplementedException();
    }

    public override string GetSysSchemaParentsInPackageHierarchySelectSqlText(bool usePackageHierarchy)
    {
        if (usePackageHierarchy)
        {
            return 
                $"""
                SELECT * 
                FROM {CurrentSchemaName}.{StartDelimiter}tsp_GetSysSchemaParentsInPackageHierarchyByPackage{EndDelimiter}(
                    {ParameterMarker}SchemaUId, 
                    {ParameterMarker}StartSchemaUId, 
                    {ParameterMarker}WorkspaceId, 
                    {ParameterMarker}SysPackageId)
                """;
        }

        return 
            $"""
            SELECT * 
            FROM {CurrentSchemaName}.{StartDelimiter}tsp_GetSysSchemaParentsInPackageHierarchyByPackage{EndDelimiter}(
                {ParameterMarker}StartSchemaUId, 
                {ParameterMarker}WorkspaceId)
            """;
    }

    public override string GetDaysOfWeekSqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetWorkspaceIdByNumberSelectSqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetAppendBlobSqlText(string entitySchemaName, string columnName)
    {
        throw new NotImplementedException();
    }

    public override bool NeedWrapFilterTextInUpperFunction(EntitySchemaQueryFilter filter)
    {
        return false;
    }

    public override bool NeedWrapFilterTextInUpperFunction()
    {
        throw new NotImplementedException();
    }

    public override void BuildDBObjectNameDataValueTypeSqlText(
        StringBuilder sb, 
        DBObjectNameDataValueType dbObjectNameDataValueType)
    {
        throw new NotImplementedException();
    }

    public override void BuildTextDataValueTypeSqlText(StringBuilder sb, TextDataValueType textDataValueType)
    {
        throw new NotImplementedException();
    }

    public override void BuildFloatDataValueTypeSqlText(StringBuilder sb, FloatDataValueType floatDataValueType)
    {
        throw new NotImplementedException();
    }

    public override void BuildBooleanDataValueTypeSqlText(StringBuilder sb, BooleanDataValueType booleanDataValueType)
    {
        throw new NotImplementedException();
    }

    public override void BuildDateTimeDataValueTypeSqlText(StringBuilder sb, DateTimeDataValueType dateTimeDataValueType)
    {
        throw new NotImplementedException();
    }

    public override void BuildGuidDataValueTypeSqlText(StringBuilder sb, GuidDataValueType guidDataValueType)
    {
        throw new NotImplementedException();
    }

    public override void BuildBinaryDataValueTypeSqlText(StringBuilder sb, BinaryDataValueType binaryDataValueType)
    {
        throw new NotImplementedException();
    }

    public override bool IsTextDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsDbObjectNameDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsIntegerDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsFloatDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsBooleanDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsDateTimeDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsGuidDataValueType(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsMaxSizeText(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override bool IsUnicodeText(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override DateTimeValueKind GetDateTimeValueKind(DbColumnDto columnDto)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(DBObjectNameDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(TextDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(IntegerDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(FloatDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(BooleanDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(DateTimeDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(TimeDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(DateDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(GuidDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override void InitializeDataValueDbData(BinaryDataValueType dataValueType)
    {
        throw new NotImplementedException();
    }

    public override string GetIsEntitySchemaExistSqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetIndexesQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetIsNotNullColumnSqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetColumnQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetColumnsQuerySqlText(bool needGetConstraintType)
    {
        throw new NotImplementedException();
    }

    public override string GetColumnListQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetConstraintsQuerySqlText()
    {
        throw new NotImplementedException();
    }

    public override string GetConstraintQuerySqlText()
    {
        throw new NotImplementedException();
    }
}