use std::path::{Path, PathBuf};

use console::style;
use nom::{bytes::complete::tag, character::complete::multispace0, IResult};

use crate::fenced;

#[derive(Debug, Clone)]
pub struct ProjectReference {
    include: String,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub path: PathBuf,
    project_references: Vec<ProjectReference>,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        let includes = extract_includes(&path);

        Self {
            path,
            project_references: includes
                .into_iter()
                .map(|include| ProjectReference { include })
                .collect(),
        }
    }

    pub fn pretty_print(&self) {
        println!(
            "Project {}, {} deps:",
            style(self.path.display()).cyan().italic(),
            style(self.project_references.len()).yellow().bold()
        );

        self.project_references
            .clone()
            .into_iter()
            .enumerate()
            .for_each(|(i, project_reference)| {
                println!(
                    "        {}: {}",
                    style(i + 1).bold(),
                    style(&project_reference.include).dim()
                );
            });
    }
}

fn extract_include(input: &str) -> IResult<&str, String> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("<ProjectReference")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("Include=")(input)?;

    let (input, path) = fenced("\"", "\"")(input)?;

    Ok((input, path.to_string()))
}

fn extract_includes(project_path: &Path) -> Vec<String> {
    let input = std::fs::read_to_string(project_path)
        .unwrap_or_else(|_| panic!("Failed to read project file at {}", project_path.display()));

    let mut includes = Vec::new();
    for line in input.lines() {
        if let Ok((_, include)) = extract_include(line) {
            includes.push(include);
        }
    }

    let mut resolved_paths = Vec::new();
    for include in includes {
        let mut path = project_path.to_path_buf();
        path.pop(); // pop from file to directory
        let components: Vec<&str> = include.split('\\').collect();
        for component in components {
            if component == ".." {
                path.pop();
            } else {
                path.push(component);
            }
        }

        resolved_paths.push(path.to_str().unwrap().to_string());
    }

    resolved_paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_includes() {
        let input = r#"
        <Project Sdk="Microsoft.NET.Sdk">
            <PropertyGroup>
                <TargetFramework>net7.0</TargetFramework>
                <AzureFunctionsVersion>v4</AzureFunctionsVersion>
                <OutputType>Exe</OutputType>
                <ImplicitUsings>enable</ImplicitUsings>
                <Nullable>enable</Nullable>
                <UserSecretsId>56c7c508-eb73-4666-a18f-459934dda267</UserSecretsId>
                <Configurations>Debug;Release;QuickDebug</Configurations>
            </PropertyGroup>
            <ItemGroup>
                <PackageReference Include="Azure.Storage.Blobs" Version="12.17.0" />
                <PackageReference Include="Azure.Storage.Files.Shares" Version="12.15.0" />
                <PackageReference Include="Azure.Storage.Queues" Version="12.15.0" />
                <PackageReference Include="Microsoft.Azure.Functions.Worker" Version="1.19.0" />
                <PackageReference Include="Microsoft.Azure.Functions.Worker.Extensions.Storage" Version="6.1.0" />
                <PackageReference Include="Microsoft.Azure.Functions.Worker.Sdk" Version="1.14.0" />
                <PackageReference Include="Microsoft.Extensions.Azure" Version="1.7.0" />
            </ItemGroup>
            <ItemGroup>
                <ProjectReference Include="..\..\SharedLibraries\Shared.Api.ServiceBus\Shared.Api.ServiceBus.csproj" />
                <ProjectReference Include="..\..\SharedLibraries\Shared.Infrastructure\Shared.Infrastructure.csproj" />
                <ProjectReference Include="..\WashingMachine\WashingMachine\WashingMachine.csproj" />
            </ItemGroup>
            <ItemGroup>
                <None Update="host.json">
                <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
                </None>
                <None Update="local.settings.json">
                <CopyToOutputDirectory>Always</CopyToOutputDirectory>
                <CopyToPublishDirectory>Never</CopyToPublishDirectory>
                </None>
            </ItemGroup>
            <ItemGroup>
                <Using Include="System.Threading.ExecutionContext" Alias="ExecutionContext" />
            </ItemGroup>
        </Project>
        "#;

        let mut includes = Vec::new();
        for line in input.lines() {
            if let Ok((_, include)) = extract_include(line) {
                includes.push(include);
            }
        }

        assert_eq!(
            includes,
            vec![
                r#"..\..\SharedLibraries\Shared.Api.ServiceBus\Shared.Api.ServiceBus.csproj"#,
                r#"..\..\SharedLibraries\Shared.Infrastructure\Shared.Infrastructure.csproj"#,
                r#"..\WashingMachine\WashingMachine\WashingMachine.csproj"#
            ]
        );
    }
}
