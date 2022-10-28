/* SPDX-License-Identifier: AGPL-3.0-only
*
* This file is part of HarTex.
*
* HarTex
* Copyright (c) 2021-2022 HarTex Project Developers
*
* HarTex is free software; you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as published by
* the Free Software Foundation; either version 3 of the License, or
* (at your option) any later version.
*
* HarTex is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License along
* with HarTex. If not, see <https://www.gnu.org/licenses/>.
*/

use futures_util::StreamExt;
use hartex_discord_core::log;
use lapin::options::BasicAckOptions;
use lapin::Consumer;
use serde_scan::scan;

pub async fn consume(mut consumer: Consumer) -> hartex_discord_eyre::Result<()> {
    while let Some(result) = consumer.next().await {
        if let Ok(delivery) = result {
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("failed to ack");
            let value = delivery.routing_key.as_str();
            let scanned: (u8, u8) = scan!("CLUSTER {} SHARD {} PAYLOAD" <- value)?;
            log::trace!(
                "[cluster {} - shard {}] {}",
                scanned.0,
                scanned.1,
                String::from_utf8(delivery.data).unwrap()
            );
        }
    }

    Ok(())
}