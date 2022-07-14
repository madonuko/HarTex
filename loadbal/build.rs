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

#![feature(let_else)]

use std::fmt::Write;
use std::fs;

use toml_edit::{Document, Item, Value};

pub fn main() {
    let result = fs::read_to_string("../buildconf.toml");
    if let Err(error) = &result {
        println!("cargo:warning=cannot open build configuration file: `{error}`; the `loadbal` (bin and lib) crates will not compile");
        return;
    }

    let config = result.unwrap();
    let config = config.trim_end().to_string();

    let result = config.parse::<Document>();
    if let Err(error) = &result {
        println!(
            "cargo:warning=invalid build configuration file: `{error}`; the `loadbal` (bin and lib) crates will not compile"
        );
        return;
    }
    let value = result.unwrap();
    let servers_item = value["loadbal"]["servers"].clone();
    let Item::Value(Value::Array(servers)) = servers_item else {
        println!(
            "cargo:warning=servers not found in configuration file; the `loadbal` (bin and lib) crates will not compile"
        );
        return;
    };
    let mut servers_env = String::from("cargo:rustc-env=LOADBAL_SERVERS=");
    servers.iter().for_each(|table_value| {
        let Value::InlineTable(table) = table_value else {
            println!(
                "cargo:warning=configuration file invalid: expected array of inline tables for servers; the `loadbal` (bin and lib) crates will not compile"
            );
            return;
        };
        let Value::String(server_type) = table["type"].clone() else {
            println!(
                "cargo:warning=configuration file invalid: expected string for server type; the `loadbal` (bin and lib) crates will not compile"
            );
            return;
        };
        let Value::String(server_address) = table["address"].clone() else {
            println!(
                "cargo:warning=configuration file invalid: expected string for server address; the `loadbal` (bin and lib) crates will not compile"
            );
            return;
        };

        write!(
            servers_env, "{}-{};",
            server_type.value().as_str(),
            server_address.value().as_str()
        ).unwrap();
    });
    println!("{servers_env}");
}