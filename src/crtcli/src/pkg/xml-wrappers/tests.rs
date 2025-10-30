use crate::pkg::xml_wrappers::{csproj, resource};

#[path = "csproj_tests_data.rs"]
mod csproj_tests_data;

#[path = "csproj_tests_data_1.rs"]
mod csproj_tests_data_1;

#[path = "resource_tests_data.rs"]
mod resource_tests_data;

#[test]
fn resource_sorting() {
    let input = resource_tests_data::INPUT.as_bytes();
    let expected_output = resource_tests_data::RESOURCE_SORTING_EXPECTED_OUTPUT.as_bytes();

    let output =
        resource::apply_sorting(input, crate::pkg::transforms::SortingComparer::Alnum).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting() {
    let input = csproj_tests_data::INPUT.as_bytes();
    let expected_output = csproj_tests_data::CSPROJ_SORTING_EXPECTED_OUTPUT.as_bytes();

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting_without_sorting_attributes() {
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

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting_without_blocks_inside_1() {
    let input = csproj_tests_data_1::INPUT.as_bytes();
    let expected_output = input; // same as input / unchanged

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting_without_blocks_inside_2() {
    let input = csproj_tests_data_1::INPUT_2.as_bytes();
    let expected_output = input; // same as input / unchanged

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting_without_blocks_inside_3() {
    let input = csproj_tests_data_1::INPUT_3.as_bytes();
    let expected_output = input; // same as input / unchanged

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}
