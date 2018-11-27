use std::{
    path::PathBuf,
    fs::File,
    fmt,
};

use rbxmx;

use crate::{
    rbx_session::construct_oneoff_tree,
    project::{Project, ProjectLoadFuzzyError},
    imfs::Imfs,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    Rbxmx,
    Rbxlx,
}

fn detect_output_kind(options: &BuildOptions) -> Option<OutputKind> {
    match options.output_kind {
        Some(output_kind) => Some(output_kind),
        None => {
            let extension = options.output_file.extension()?.to_str()?;

            match extension {
                "rbxlx" => Some(OutputKind::Rbxlx),
                "rbxmx" => Some(OutputKind::Rbxmx),
                _ => None,
            }
        },
    }
}

#[derive(Debug)]
pub struct BuildOptions {
    pub fuzzy_project_path: PathBuf,
    pub output_file: PathBuf,
    pub output_kind: Option<OutputKind>,
}

pub enum BuildError {
    UnknownOutputKind,
    ProjectLoadError(ProjectLoadFuzzyError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildError::UnknownOutputKind => {
                write!(output, "Could not detect what kind of output file to create")
            },
            BuildError::ProjectLoadError(inner) => write!(output, "{}", inner),
        }
    }
}

impl From<ProjectLoadFuzzyError> for BuildError {
    fn from(error: ProjectLoadFuzzyError) -> BuildError {
        BuildError::ProjectLoadError(error)
    }
}

pub fn build(options: &BuildOptions) -> Result<(), BuildError> {
    let output_kind = detect_output_kind(options)
        .ok_or(BuildError::UnknownOutputKind)?;

    info!("Hoping to generate file of type {:?}", output_kind);

    info!("Looking for project at {}", options.fuzzy_project_path.display());

    let project = Project::load_fuzzy(&options.fuzzy_project_path)?;

    info!("Found project at {}", project.file_location.display());
    info!("Using project {:#?}", project);

    let imfs = Imfs::new(&project)
        .expect("Could not create in-memory filesystem");

    let tree = construct_oneoff_tree(&project, &imfs);

    let mut file = File::create(&options.output_file)
        .expect("Could not open output file for write");

    match output_kind {
        OutputKind::Rbxmx => {
            let root_id = tree.get_root_id();
            rbxmx::encode(&tree, &[root_id], &mut file);
        },
        OutputKind::Rbxlx => {
            let root_id = tree.get_root_id();
            let top_level_ids = tree.get_instance(root_id).unwrap().get_children_ids();
            rbxmx::encode(&tree, top_level_ids, &mut file);
        },
    }

    Ok(())
}