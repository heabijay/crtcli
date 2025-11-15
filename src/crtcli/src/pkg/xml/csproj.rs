use crate::pkg::xml::*;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use regex::Regex;
use std::cmp::Ordering;
use std::io::Cursor;
use std::sync::LazyLock;
use thiserror::Error;

const TERRASOFT_CONFIGURATION: &str = "Terrasoft.Configuration";

const TERRASOFT_CONFIGURATION_REFERENCE_BLOCK: &str =
    "Reference Include=\"Terrasoft.Configuration\"";

const TERRASOFT_CONFIGURATION_HINT_PATH: &str =
    "$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll";

pub static PKG_CSPROJ_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Files{sep}[^{sep}]+\.csproj$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package csproj file path regex")
});

#[derive(Debug, Error)]
pub enum CsprojProcessingError {
    #[error("error reading csproj xml: {0}")]
    Reader(#[from] quick_xml::Error),

    #[error("failed to process content of package references block: {0}")]
    PackageReferencesBlockProcessing(#[from] XmlReadEventBlockError),

    #[error("cannot find Package References ItemGroup")]
    CannotFindPackageReferencesBlock,
}

pub fn apply_sorting(content: &[u8]) -> Result<Vec<u8>, CsprojProcessingError> {
    let mut reader = Reader::from_reader(content);
    let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

    loop {
        match reader.read_event().map_err(CsprojProcessingError::Reader)? {
            Event::Start(e) if is_package_references_item_group_start(&e) => {
                process_blocks_sorting(&mut reader, &mut writer, e)?;
            }
            Event::Eof => break,
            e => writer.write_event(e).unwrap(),
        }
    }

    return Ok(writer.into_inner().into_inner());

    fn process_blocks_sorting(
        reader: &mut Reader<&[u8]>,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        start_tag: BytesStart,
    ) -> Result<(), XmlReadEventBlockError> {
        writer.write_event(Event::Start(start_tag)).unwrap();

        let (mut event_blocks, end_tag) = xml_read_event_blocks(reader)?;

        event_blocks.blocks.sort_by(|a, b| {
            let a = a.first().expect("event block cannot be empty");
            let b = b.first().expect("event block cannot be empty");

            if a.eq_ignore_ascii_case(TERRASOFT_CONFIGURATION_REFERENCE_BLOCK.as_ref()) {
                Ordering::Less
            } else if b.eq_ignore_ascii_case(TERRASOFT_CONFIGURATION_REFERENCE_BLOCK.as_ref()) {
                Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        if let Some(ref open_sep) = event_blocks.separator {
            writer.write_event(Event::Text(open_sep.clone())).unwrap();
        }

        xml_write_event_blocks(writer, event_blocks);

        writer.write_event(Event::End(end_tag)).unwrap();

        Ok(())
    }
}

pub fn modify_package_references(
    content: &[u8],
    add_configuration_dll: bool,
    pkg_names: &[impl AsRef<str>],
) -> Result<Vec<u8>, CsprojProcessingError> {
    let mut reader = Reader::from_reader(content);
    let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
    let mut package_references_item_group_found = false;
    let mut previous_event_is_end_tag = false;
    let mut indent = None;

    loop {
        let event = reader.read_event().map_err(CsprojProcessingError::Reader)?;

        if let Event::End(_) = event {
            previous_event_is_end_tag = true;
        } else if previous_event_is_end_tag && let Event::Text(text) = event.as_ref() {
            indent = Some(text.clone());
        } else {
            previous_event_is_end_tag = false;
        }

        match event {
            Event::Eof => break,
            Event::Start(ref e) | Event::Empty(ref e)
                if is_package_references_item_group_start(e) =>
            {
                if matches!(event, Event::Start(_)) {
                    xml_skip_event_blocks(&mut reader)?;
                }

                package_references_item_group_found = true;

                write_package_references_item_group(
                    &mut writer,
                    indent.clone(),
                    add_configuration_dll,
                    pkg_names,
                )?;
            }
            e => writer.write_event(e).unwrap(),
        }
    }

    if !package_references_item_group_found {
        return Err(CsprojProcessingError::CannotFindPackageReferencesBlock);
    }

    return Ok(writer.into_inner().into_inner());

    fn write_package_references_item_group(
        writer: &mut Writer<Cursor<Vec<u8>>>,
        indent: Option<BytesText>,
        add_configuration_dll: bool,
        pkg_names: &[impl AsRef<str>],
    ) -> Result<(), XmlReadEventBlockError> {
        writer
            .write_event(Event::Start({
                let mut start_tag = BytesStart::new("ItemGroup");
                start_tag.push_attribute(("Label", "Package References"));
                start_tag
            }))
            .unwrap();

        if !add_configuration_dll && pkg_names.is_empty() {
            writer
                .write_event(Event::End(BytesEnd::new("ItemGroup")))
                .unwrap();

            return Ok(());
        }

        if add_configuration_dll {
            write_indent_if_present(writer, indent.clone(), 1);

            write_std_package_reference(
                writer,
                TERRASOFT_CONFIGURATION,
                TERRASOFT_CONFIGURATION_HINT_PATH,
                indent.as_ref(),
            );
        }

        for pkg_name in pkg_names.iter() {
            write_indent_if_present(writer, indent.clone(), 1);

            let pkg_name = pkg_name.as_ref();

            write_std_package_reference(
                writer,
                pkg_name,
                &format!(
                    "$(RelativePkgFolderPath)/{pkg_name}/$(StandalonePackageAssemblyPath)/{pkg_name}.dll"
                ),
                indent.as_ref(),
            );
        }

        write_indent_if_present(writer, indent, 0);

        writer
            .write_event(Event::End(BytesEnd::new("ItemGroup")))
            .unwrap();

        return Ok(());

        fn write_std_package_reference(
            writer: &mut Writer<Cursor<Vec<u8>>>,
            name: &str,
            hint_path: &str,
            base_indent: Option<&BytesText>,
        ) {
            let mut start_reference_el = BytesStart::new("Reference");
            let end_reference_el = BytesEnd::new("Reference");

            start_reference_el.push_attribute(("Include", name));

            writer
                .write_event(Event::Start(start_reference_el))
                .unwrap();

            write_indent_if_present(writer, base_indent.cloned(), 2);
            write_simple_value(writer, "HintPath", hint_path);

            write_indent_if_present(writer, base_indent.cloned(), 2);
            write_simple_value(writer, "SpecificVersion", "False");

            write_indent_if_present(writer, base_indent.cloned(), 2);
            write_simple_value(writer, "Private", "False");

            write_indent_if_present(writer, base_indent.cloned(), 1);
            writer.write_event(Event::End(end_reference_el)).unwrap();
        }
    }
}

fn is_item_group_start(e: &BytesStart) -> bool {
    e.name().as_ref() == b"ItemGroup"
}

fn is_package_references_item_group_start(e: &BytesStart) -> bool {
    is_item_group_start(e)
        && e.try_get_attribute("Label")
            .is_ok_and(|x| x.is_some_and(|x| x.value.as_ref() == b"Package References"))
}

fn write_indent_if_present(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    base_indent: Option<BytesText>,
    level: u32,
) {
    let Some(base_indent) = base_indent else {
        return;
    };

    writer.write_event(Event::Text(base_indent)).unwrap();

    for _ in 0..level {
        writer
            .write_event(Event::Text(BytesText::new("\t")))
            .unwrap();
    }
}

fn write_simple_value(writer: &mut Writer<Cursor<Vec<u8>>>, key: &str, value: &str) {
    // <key>value</key>

    let tag = BytesStart::new(key);
    let value = BytesText::new(value);
    let tag_end = BytesEnd::new(key);

    writer.write_event(Event::Start(tag)).unwrap();
    writer.write_event(Event::Text(value)).unwrap();
    writer.write_event(Event::End(tag_end)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorting() {
        let input = tests_data::csproj_tests_data::INPUT.as_bytes();
        let expected_output =
            tests_data::csproj_tests_data::CSPROJ_SORTING_EXPECTED_OUTPUT.as_bytes();

        let output = apply_sorting(input).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn sorting_without_sorting_attributes() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
		</Project>
		"#;

        let expected_output = input;

        let output = apply_sorting(input).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn sorting_without_blocks_inside_1() {
        let input = tests_data::csproj_tests_data_1::INPUT.as_bytes();
        let expected_output = input; // same as input / unchanged

        let output = apply_sorting(input).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn sorting_without_blocks_inside_2() {
        let input = tests_data::csproj_tests_data_1::INPUT_2.as_bytes();
        let expected_output = input; // same as input / unchanged

        let output = apply_sorting(input).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn sorting_without_blocks_inside_3() {
        let input = tests_data::csproj_tests_data_1::INPUT_3.as_bytes();
        let expected_output = input; // same as input / unchanged

        let output = apply_sorting(input).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references() {
        let input = tests_data::csproj_tests_data::INPUT.as_bytes();
        let expected_output =
            tests_data::csproj_tests_data::CSPROJ_MODIFY_PACKAGE_REFERENCES_EXPECTED_OUTPUT
                .as_bytes();

        let add_configuration_dll = true;
        let packages = ["CrtBaseConsts", "CrtCore", "CrtCoreBase", "SsoSettings"];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_no_block() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
		</Project>
		"#;

        let add_configuration_dll = true;
        let packages = ["CrtBaseConsts", "CrtCore", "CrtCoreBase", "SsoSettings"];

        let output = super::modify_package_references(input, add_configuration_dll, &packages);

        assert!(matches!(
            output,
            Err(CsprojProcessingError::CannotFindPackageReferencesBlock)
        ))
    }

    #[test]
    fn modify_package_references_simple() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtBaseConsts">
					<HintPath>$(RelativePkgFolderPath)/CrtBaseConsts/$(StandalonePackageAssemblyPath)/CrtBaseConsts.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCore">
					<HintPath>$(RelativePkgFolderPath)/CrtCore/$(StandalonePackageAssemblyPath)/CrtCore.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCoreBase">
					<HintPath>$(RelativePkgFolderPath)/CrtCoreBase/$(StandalonePackageAssemblyPath)/CrtCoreBase.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="SsoSettings">
					<HintPath>$(RelativePkgFolderPath)/SsoSettings/$(StandalonePackageAssemblyPath)/SsoSettings.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = true;
        let packages = ["CrtBaseConsts", "CrtCore", "CrtCoreBase", "SsoSettings"];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_simple_no_packages() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References"></ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = false;
        let packages: [&str; 0] = [];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_simple_only_configuration_dll() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = true;
        let packages: [&str; 0] = [];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_simple_without_configuration_dll() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="CrtBaseConsts">
					<HintPath>$(RelativePkgFolderPath)/CrtBaseConsts/$(StandalonePackageAssemblyPath)/CrtBaseConsts.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCore">
					<HintPath>$(RelativePkgFolderPath)/CrtCore/$(StandalonePackageAssemblyPath)/CrtCore.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCoreBase">
					<HintPath>$(RelativePkgFolderPath)/CrtCoreBase/$(StandalonePackageAssemblyPath)/CrtCoreBase.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="SsoSettings">
					<HintPath>$(RelativePkgFolderPath)/SsoSettings/$(StandalonePackageAssemblyPath)/SsoSettings.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = false;
        let packages = ["CrtBaseConsts", "CrtCore", "CrtCoreBase", "SsoSettings"];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_empty() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References"></ItemGroup>
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtBaseConsts">
					<HintPath>$(RelativePkgFolderPath)/CrtBaseConsts/$(StandalonePackageAssemblyPath)/CrtBaseConsts.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCore">
					<HintPath>$(RelativePkgFolderPath)/CrtCore/$(StandalonePackageAssemblyPath)/CrtCore.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCoreBase">
					<HintPath>$(RelativePkgFolderPath)/CrtCoreBase/$(StandalonePackageAssemblyPath)/CrtCoreBase.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="SsoSettings">
					<HintPath>$(RelativePkgFolderPath)/SsoSettings/$(StandalonePackageAssemblyPath)/SsoSettings.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = true;
        let packages = ["CrtBaseConsts", "CrtCore", "CrtCoreBase", "SsoSettings"];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_empty_closed() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References" />
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References">
				<Reference Include="Terrasoft.Configuration">
					<HintPath>$(RelativePkgFolderPath)/../bin/Terrasoft.Configuration.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtBaseConsts">
					<HintPath>$(RelativePkgFolderPath)/CrtBaseConsts/$(StandalonePackageAssemblyPath)/CrtBaseConsts.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCore">
					<HintPath>$(RelativePkgFolderPath)/CrtCore/$(StandalonePackageAssemblyPath)/CrtCore.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="CrtCoreBase">
					<HintPath>$(RelativePkgFolderPath)/CrtCoreBase/$(StandalonePackageAssemblyPath)/CrtCoreBase.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
				<Reference Include="SsoSettings">
					<HintPath>$(RelativePkgFolderPath)/SsoSettings/$(StandalonePackageAssemblyPath)/SsoSettings.dll</HintPath>
					<SpecificVersion>False</SpecificVersion>
					<Private>False</Private>
				</Reference>
			</ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = true;
        let packages = ["CrtBaseConsts", "CrtCore", "CrtCoreBase", "SsoSettings"];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn modify_package_references_empty_no_packages() {
        let input = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References"></ItemGroup>
		</Project>
		"#;

        let expected_output = br#"
		<Project Sdk="Microsoft.NET.Sdk">
			<PropertyGroup>
				<AppendTargetFrameworkToOutputPath>False</AppendTargetFrameworkToOutputPath>
				<CoreTargetFramework Condition="'$(CoreTargetFramework)' == ''">net472</CoreTargetFramework>
				<TargetFramework>$(CoreTargetFramework)</TargetFramework>
			</PropertyGroup>
			<ItemGroup Label="Package References"></ItemGroup>
		</Project>
		"#;

        let add_configuration_dll = false;
        let packages: [&str; 0] = [];

        let output =
            super::modify_package_references(input, add_configuration_dll, &packages).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }
}
