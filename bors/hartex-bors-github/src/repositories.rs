/*
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * This file is part of HarTex.
 *
 * HarTex
 * Copyright (c) 2021-2023 HarTex Project Developers
 *
 * HarTex is free software; you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation; either version 3 of the License, or
 * (at your option) any later version.
 *
 * HarTex is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License along
 * with HarTex. If not, see <https://www.gnu.org/licenses/>.
 */

//! # Repository Loading

use std::collections::HashMap;

use hartex_eyre::eyre::Report;
use hartex_log::log;
use octocrab::models::InstallationRepositories;
use octocrab::models::Repository;
use octocrab::Octocrab;
use hartex_bors_core::models::GithubRepositoryName;
use hartex_bors_permissions::BackendApiPermissionResolver;

use crate::GithubRepositoryClient;
use crate::RepositoryMap;
use crate::RepositoryState;

async fn create_repository_state(
    repository_client: Octocrab,
    repository: Repository,
) -> hartex_eyre::Result<RepositoryState> {
    let Some(owner) = repository.owner else {
        return Err(Report::msg(format!("repository {} has no owner", repository.name)));
    };

    let name = GithubRepositoryName::new(&owner.login, &repository.name);
    log::info!("found repository {name}");

    let permission_resolver = BackendApiPermissionResolver::load(name.clone()).await?;
    let client = GithubRepositoryClient {
        client,
        repository_name,
        repository,
    };

    Ok(RepositoryState {
        repository: name,
        client,
        permission_resolver: Box::new(permission_resolver),
    })
}

pub(crate) async fn load_repositories(client: &Octocrab) -> hartex_eyre::Result<RepositoryMap> {
    let installations = client.apps().installations().send().await?;

    let mut hashmap = HashMap::new();
    for installation in installations {
        if let Some(ref url) = installation.repositories_url {
            let installation_client = client.installation(installation.id);

            match installation_client
                .get::<InstallationRepositories, _, ()>(url, None)
                .await
            {
                Ok(repositories) => {
                    for repository in repositories.repositories {
                        let state =
                            create_repository_state(installation_client, repository).await?;
                        log::info!("repository loaded: {}", state.repository);

                        if let Some(existing_repository) =
                            hashmap.insert(state.repository.clone(), state)
                        {
                            return Err(Report::msg(format!(
                                "repository {} found in multiple installations",
                                state.repository
                            )));
                        }
                    }
                }
                Err(error) => {
                    return Err(Report::new(error));
                }
            }
        }
    }

    Ok(hashmap)
}
