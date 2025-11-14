use crate::pkg::json::{
    PkgJsonWrapper, PkgJsonWrapperCreateError, PkgPackageDescriptorJsonWrapper,
};
use crate::pkg::paths;
use crate::pkg::transforms::post::PkgFolderPostTransform;
use anstyle::{AnsiColor, Color, Style};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub struct CsprojPkgRefsRegeneratePkgFolderPostTransform {}

impl CsprojPkgRefsRegeneratePkgFolderPostTransform {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Error, Debug)]
pub enum CsprojPkgRefsRegeneratePkgFolderPostTransformError {
    #[error("cannot read package descriptor: {0}")]
    PkgJsonWrapperCreate(#[from] PkgJsonWrapperCreateError),

    #[error("failed to read {0}: {1}")]
    Read(PathBuf, #[source] std::io::Error),

    #[error("failed to process {0}: {1}")]
    ProcessCsproj(
        PathBuf,
        #[source] crate::pkg::xml::csproj::CsprojProcessingError,
    ),

    #[error("failed to write {0}: {1}")]
    Write(PathBuf, #[source] std::io::Error),
}

impl PkgFolderPostTransform for CsprojPkgRefsRegeneratePkgFolderPostTransform {
    type Error = CsprojPkgRefsRegeneratePkgFolderPostTransformError;

    fn transform(&self, pkg_folder: &Path, check_only: bool) -> Result<bool, Self::Error> {
        let descriptor = PkgPackageDescriptorJsonWrapper::from(PkgJsonWrapper::from_file(
            &pkg_folder.join(paths::PKG_DESCRIPTOR_FILE),
        )?);

        let Some(project_path) = descriptor.project_path() else {
            println!(
                "{style}warning: $.Descriptor.ProjectPath value was not found in descriptor.json. Skipping csproj pkg refs regeneration{style:#}",
                style = Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                    .dimmed(),
            );

            return Ok(false);
        };

        let csproj_path = pkg_folder.join(project_path);

        let dependent_pkgs = descriptor.depends_on().unwrap_or_default();
        let depend_on_std_pkg = dependent_pkgs.iter().any(|x| x.pkg_type() == 0);

        let mut asm_pkg_names: Vec<&str> = dependent_pkgs
            .iter()
            .filter(|x| x.pkg_type() == 1)
            .filter_map(|x| x.name())
            .collect();

        asm_pkg_names.sort();

        let source_content = std::fs::read(&csproj_path).map_err(|err| {
            CsprojPkgRefsRegeneratePkgFolderPostTransformError::Read(csproj_path.clone(), err)
        })?;

        let result_content = crate::pkg::xml::csproj::modify_package_references(
            &source_content,
            depend_on_std_pkg,
            &asm_pkg_names,
        )
        .map_err(|err| {
            CsprojPkgRefsRegeneratePkgFolderPostTransformError::ProcessCsproj(
                csproj_path.clone(),
                err,
            )
        })?;

        if source_content == result_content {
            Ok(false)
        } else {
            if check_only {
                println!("\tto regenerate:\t{}", project_path);
            } else {
                std::fs::write(&csproj_path, result_content).map_err(|err| {
                    CsprojPkgRefsRegeneratePkgFolderPostTransformError::Write(csproj_path, err)
                })?;

                println!("\tregenerated:\t{}", project_path);
            }

            Ok(true)
        }
    }
}
