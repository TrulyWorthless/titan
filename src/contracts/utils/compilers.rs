use ethers_solc::{Project, ProjectCompileOutput, ProjectPathsConfig};
use eyre::Result;

pub async fn basic() -> Result<ProjectCompileOutput> {
    let project: Project = Project::builder().build().unwrap();
    let output: ProjectCompileOutput = project.compile().unwrap();

    Ok(output)
}

pub async fn compile_solidity() -> Result<ProjectCompileOutput> {
    let project: Project = Project::builder()
        .paths(ProjectPathsConfig::hardhat(env!("CARGO_MANIFEST_DIR")).unwrap())
        .build()
        .unwrap();

    let output: ProjectCompileOutput = project.compile().unwrap();
    project.rerun_if_sources_changed();

    Ok(output)
}

pub async fn compile_solidity_advanced() -> Result<ProjectCompileOutput> {
    let project: Project = Project::builder().build().unwrap();
    let output: ProjectCompileOutput = project
        .compile_files(vec!["examples/Foo.sol", "examples/Bar.sol"])
        .unwrap();

    Ok(output)
}
