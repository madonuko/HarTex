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

use std::env;

use hartex_database_queries::discord_frontend::queries::cached_user_select_by_id::cached_user_select_by_id;
use hartex_database_queries::discord_frontend::queries::cached_user_upsert::cached_user_upsert;
use hartex_discord_entitycache_core::error::CacheResult;
use hartex_discord_entitycache_core::traits::Entity;
use hartex_discord_entitycache_core::traits::Repository;
use hartex_discord_entitycache_entities::user::UserEntity;
use tokio_postgres::NoTls;

pub struct CachedUserRepository;

impl Repository<UserEntity> for CachedUserRepository {
    async fn get(&self, id: <UserEntity as Entity>::Id) -> CacheResult<UserEntity> {
        let (client, _) = tokio_postgres::connect(&env::var("HARTEX_NIGHTLY_PGSQL_URL")?, NoTls).await?;

        let data = cached_user_select_by_id()
            .bind(&client, &id.to_string())
            .one()
            .await?;

        Ok(UserEntity { bot: data.bot, id })
    }

    async fn upsert(&self, entity: UserEntity) -> CacheResult<()> {
        let (client, _) = tokio_postgres::connect(&env::var("HARTEX_NIGHTLY_PGSQL_URL")?, NoTls).await?;

        cached_user_upsert()
            .bind(&client, &entity.id.to_string(), &entity.bot)
            .await?;

        Ok(())
    }
}
