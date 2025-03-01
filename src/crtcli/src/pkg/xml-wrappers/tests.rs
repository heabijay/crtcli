use crate::pkg::xml_wrappers::csproj;

#[path = "tests_data.rs"]
mod tests_data;

#[path = "tests_data_1.rs"]
mod tests_data_1;

#[test]
fn csproj_sorting() {
    let input = tests_data::INPUT.as_bytes();
    let expected_output = tests_data::CSPROJ_SORTING_EXPECTED_OUTPUT.as_bytes();

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
    let input = tests_data_1::INPUT.as_bytes();
    let expected_output = input; // same as input / unchanged

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting_without_blocks_inside_2() {
    let input = tests_data_1::INPUT_2.as_bytes();
    let expected_output = input; // same as input / unchanged

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn csproj_sorting_without_blocks_inside_3() {
    let input = tests_data_1::INPUT_3.as_bytes();
    let expected_output = input; // same as input / unchanged

    let output = csproj::apply_sorting(input).unwrap();

    pretty_assertions::assert_eq!(
        String::from_utf8_lossy(expected_output),
        String::from_utf8_lossy(&output)
    );
}
