pub const INPUT: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<Resources Culture="en-US">
	<Group Type="String">
		<Items>
			<Item Name="BaseElements.AddDataUserTask1.Caption" Value="1 way" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.RecordId.Caption" Value="Created record Id" />
		</Items>
	</Group>
</Resources>
"#;

pub const RESOURCE_SORTING_EXPECTED_OUTPUT: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<Resources Culture="en-US">
	<Group Type="String">
		<Items>
			<Item Name="BaseElements.AddDataUserTask1.Caption" Value="1 way" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask1.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask2.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask3.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask4.Parameters.RecordId.Caption" Value="Created record Id" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.DataSourceFilters.Caption" Value="Data source filters" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.EntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.FilterEntitySchemaId.Caption" Value="Object" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.RecordAddMode.Caption" Value="Record adding mode" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.RecordDefValues.Caption" Value="Set column values" />
			<Item Name="BaseElements.AddDataUserTask5.Parameters.RecordId.Caption" Value="Created record Id" />
		</Items>
	</Group>
</Resources>
"#;
