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

//! # Try cancel command
//!
//! bors try-

use hartex_bors_core::models::BorsBuild;
use hartex_bors_core::models::BorsBuildStatus;
use hartex_bors_core::models::BorsWorkflowStatus;
use hartex_bors_core::models::BorsWorkflowType;
use hartex_bors_core::models::GithubRepositoryState;
use hartex_bors_core::models::Permission;
use hartex_bors_core::DatabaseClient;
use hartex_bors_core::RepositoryClient;
use hartex_log::log;

use crate::permissions::check_permissions;

/// Executes the try cancel command.
pub async fn try_cancel_command<C: RepositoryClient>(
    repository: &mut GithubRepositoryState<C>,
    database: &mut dyn DatabaseClient,
    pr: u64,
    author: &str,
) -> hartex_eyre::Result<()> {
    if !check_permissions(repository, pr, author, Permission::TryBuild).await? {
        return Ok(());
    }

    let github_pr = repository.client.get_pull_request(pr).await?;

    let pull_request = database
        .get_or_create_pull_request(repository.client.repository_name(), None, &github_pr, pr)
        .await?;
    let Some(build) = pull_request
        .try_build
        .and_then(|build| (build.status == BorsBuildStatus::Pending).then_some(build)) else {
        repository.client.post_comment(pr, ":warning: There is currently no try build in progress.").await?;

        return Ok(());
    };

    if let Err(error) = cancel_build_workflows(repository, database, &build).await {
        println!("{error}");
    }

    database
        .update_build_status(&build, BorsBuildStatus::Cancelled)
        .await?;

    log::warn!("try build workflow cancelled");
    repository
        .client
        .post_comment(pr, ":white_check_mark: Try build cancelled.")
        .await?;

    Ok(())
}

async fn cancel_build_workflows<C: RepositoryClient>(
    repository: &mut GithubRepositoryState<C>,
    database: &mut dyn DatabaseClient,
    build: &BorsBuild,
) -> hartex_eyre::Result<()> {
    let pending_workflows = database
        .get_workflows_for_build(build)
        .await?
        .into_iter()
        .filter(|workflow| {
            workflow.workflow_status == BorsWorkflowStatus::Pending
                && workflow.workflow_type == BorsWorkflowType::GitHub
        })
        .map(|workflow| workflow.run_id)
        .collect::<Vec<_>>();

    log::trace!("cancelling workflows: {:?}", &pending_workflows);
    repository.client.cancel_workflows(pending_workflows).await
}
