use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use console::style;
use nom::{bytes::complete::take_until, IResult};

use crate::{fenced, project::Project};

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub path: PathBuf,
    pub name: String,
    pub projects: Vec<Project>,
}

impl Pipeline {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let projects = extract_projects(&path);

        Self {
            name,
            path,
            projects,
        }
    }

    pub fn complete_path_filter(&self) -> String {
        let mut dependencies = HashSet::new();
        for project in &self.projects {
            dependencies.insert(project.azure_path_filter());

            let related_projects = deep_search_for_projects(project);
            for project in related_projects {
                dependencies.insert(project.azure_path_filter());
            }
        }

        let mut dependencies = dependencies.into_iter().collect::<Vec<_>>();
        dependencies.sort();
        dependencies.into_iter().collect::<Vec<_>>().join(" ")
    }

    pub fn pretty_print(&self) {
        println!(
            "Pipeline {}, {} projects:",
            style(self.name.clone()).green().italic().bold(),
            style(self.projects.len()).yellow().bold()
        );

        self.projects.clone().into_iter().for_each(|project| {
            print!("    ");
            project.pretty_print();
        });

        println!(
            "    Path filter: {}",
            style(self.complete_path_filter()).cyan().italic()
        );
    }
}

fn extract_projects(path: &Path) -> Vec<Project> {
    let pipeline_contents = std::fs::read_to_string(path).unwrap();

    let mut base_path = PathBuf::new();
    let parts = path.components().collect::<Vec<_>>();
    for part in parts {
        if part.as_os_str() == "SE-CustomerPortal" {
            base_path.push(part.as_os_str());
            break;
        }

        base_path.push(part.as_os_str());
    }

    let mut projects = Vec::new();
    for line in pipeline_contents.lines() {
        // We don't care about the tests!
        if line.contains("Tests.csproj") || line.contains("Test.csproj") || line.contains(".Test") {
            continue;
        }

        if let Ok((_, project_path)) = extract_project_path(line, "csproj", "\"") {
            let combined_path = base_path.join(project_path);
            projects.push(Project::new(combined_path));
        } else if let Ok((_, project_path)) = extract_project_path(line, "csproj", "'") {
            let combined_path = base_path.join(project_path);
            projects.push(Project::new(combined_path));
        }
    }

    projects
}

fn extract_project_path<'a>(
    input: &'a str,
    project_extension: &str,
    fence_char: &'a str,
) -> IResult<&'a str, String> {
    let (input, _) = take_until(fence_char)(input)?;

    let (input, path) = fenced(fence_char, fence_char)(input)?;

    match PathBuf::from(path).extension() {
        Some(extension) if extension == project_extension => Ok((input, path.to_string())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn deep_search_for_projects(project: &Project) -> Vec<Project> {
    let mut projects = Vec::new();

    for reference in &project.references {
        let mut path = project.path.to_path_buf();
        path.pop(); // pop from file to directory
        path.push(&reference.include_path);

        let project = Project::new(path);
        projects.push(project);
    }

    projects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_includes() {
        let input = r#"
        trigger:
  branches:
    include:
      - main
  paths:
    include:
      - CustomerPortal/apis/modules/RentalModule

variables:
  buildConfiguration: "Release"
  projectPath: "CustomerPortal/apis/modules/RentalModule/RentalModule.Api/RentalModule.Api.csproj"
  eventSubscriberPath: "CustomerPortal/apis/modules/RentalModule/RentalModule.EventSubscribers/RentalModule.EventSubscribers.csproj"
  NUGET_PACKAGES: $(Pipeline.Workspace)/.nuget/packages
  SNAPSHOOTER_STRICT_MODE: "on"

stages:
  - stage: Build
    displayName: "Build"
    jobs:
      - job: RentalModuleApi
        pool:
          vmImage: "windows-latest"
        steps:
          # - task: PowerShell@2
          #   displayName: 'start mssqllocaldb'
          #   inputs:
          #     targetType: 'inline'
          #     script: 'sqllocaldb start mssqllocaldb'
          - task: DotNetCoreCLI@2
            displayName: "dotnet build"
            inputs:
              command: build
              projects: "$(projectPath)"
              arguments: "--configuration $(buildConfiguration)"
          - task: DotNetCoreCLI@2
            displayName: "dotnet publish"
            inputs:
              command: publish
              publishWebProjects: false
              projects: "$(projectPath)"
              arguments: "--configuration $(BuildConfiguration) --output $(Build.ArtifactStagingDirectory)"
              zipAfterPublish: True

          - task: DotNetCoreCLI@2
            displayName: "Build EventSubscriber"
            inputs:
              command: build
              projects: "$(eventSubscriberPath)"
              arguments: "--configuration $(buildConfiguration)"
          - task: DotNetCoreCLI@2
            displayName: "EventSubscriber publish"
            inputs:
              command: publish
              publishWebProjects: false
              projects: "$(eventSubscriberPath)"
              arguments: "--configuration $(BuildConfiguration) --output $(Build.ArtifactStagingDirectory)"
              zipAfterPublish: True    
          - task: PublishPipelineArtifact@1
            displayName: "Publish Artifact"
            inputs:
              targetPath: "$(Build.ArtifactStagingDirectory)"
              artifactName: "RentalModuleApi"

  - stage: DeployToDev
    displayName: "Deploy to Dev"
    dependsOn: Build
    condition: and(succeeded(), ne(variables['Build.Reason'], 'PullRequest'))
    jobs:
      - deployment: DeployToDev
        environment: "dev-customer-portal"
        displayName: Deploy to Dev
        strategy:
          runOnce:
            deploy:
              steps:
                - task: AzureWebApp@1
                  inputs:
                    azureSubscription: "DEV - ramise-customerportal-dev-rg"
                    appType: "webApp"
                    appName: "app-rentalmodule-dev"
                    package: "$(Pipeline.Workspace)/RentalModuleApi/RentalModule.Api.zip"
                    deploymentMethod: "auto"

                - task: AzureFunctionApp@2
                  inputs:
                    azureSubscription: "DEV - ramise-customerportal-dev-rg"
                    appType: "functionApp"
                    appName: "fnapp-eventsubscribers-rentalmodule-dev"
                    package: "$(Pipeline.Workspace)/RentalModuleApi/RentalModule.EventSubscribers.zip"
                    deploymentMethod: "auto"  

  - stage: DeployToUat
    displayName: "Deploy to UAT"
    dependsOn: DeployToDev
    condition: and(succeeded(), ne(variables['Build.Reason'], 'PullRequest'))
    jobs:
      - deployment: DeployToUat
        environment: "uat-customer-portal"
        displayName: Deploy to UAT
        strategy:
          runOnce:
            deploy:
              steps:
                - task: AzureWebApp@1
                  inputs:
                    azureSubscription: "UAT - ramise-customerportal-uat-rg"
                    appType: "webApp"
                    appName: "app-rentalmodule-uat"
                    package: "$(Pipeline.Workspace)/RentalModuleApi/RentalModule.Api.zip"
                    deploymentMethod: "auto"

                - task: AzureFunctionApp@2
                  inputs:
                    azureSubscription: "UAT - ramise-customerportal-uat-rg"
                    appType: "functionApp"
                    appName: "fnapp-eventsubscribers-rentalmodule-uat"
                    package: "$(Pipeline.Workspace)/RentalModuleApi/RentalModule.EventSubscribers.zip"
                    deploymentMethod: "auto"  

  - stage: DeployToProd
    displayName: "Deploy to Prod"
    dependsOn: DeployToUat
    condition: and(succeeded(), ne(variables['Build.Reason'], 'PullRequest'))
    jobs:
      - deployment: DeployToProd
        environment: "prod-customer-portal"
        displayName: Deploy to Prod
        strategy:
          runOnce:
            deploy:
              steps:
                - task: AzureWebApp@1
                  inputs:
                    azureSubscription: "PROD - ramise-customerportal-prod-rg"
                    appType: "webApp"
                    appName: "app-rentalmodule-prod"
                    package: "$(Pipeline.Workspace)/RentalModuleApi/RentalModule.Api.zip"
                    deploymentMethod: "auto"

                - task: AzureFunctionApp@2
                  inputs:
                    azureSubscription: "PROD - ramise-customerportal-prod-rg"
                    appType: "functionApp"
                    appName: "fnapp-eventsubscribers-rentalmodule-prod"
                    package: "$(Pipeline.Workspace)/RentalModuleApi/RentalModule.EventSubscribers.zip"
                    deploymentMethod: "auto" 

        "#;

        let mut projects = Vec::new();
        for line in input.lines() {
            if let Ok((_, include)) = extract_project_path(line, "csproj", "\"") {
                projects.push(include);
            }
        }

        assert_eq!(
            projects,
            vec![
                r#"CustomerPortal/apis/modules/RentalModule/RentalModule.Api/RentalModule.Api.csproj"#,
                r#"CustomerPortal/apis/modules/RentalModule/RentalModule.EventSubscribers/RentalModule.EventSubscribers.csproj"#
            ]
        );
    }
}
