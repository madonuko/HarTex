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

use proc_macro::{Delimiter, Span, TokenStream, TokenTree};

use crate::internal::StreamParser;

const VALID_ATTR_PARAMETER_NAMES: [&'static str; 4] =
    ["description", "interaction_only", "name", "type"];
const BOOLEAN_PARAMETERS: [&'static str; 1] = ["interaction_only"];
const LITERAL_PARAMETERS: [&'static str; 3] = ["description", "name", "type"];

#[derive(Debug)]
pub enum DeriveAttribute {
    Description(String),
    InteractionOnly(bool),
    Name(String),
    Type(u8),
}

#[derive(Debug)]
pub struct DeriveStream {
    pub attributes: Vec<DeriveAttribute>,
    pub item_name: String,
}

impl DeriveStream {
    fn new() -> Self {
        Self {
            attributes: Vec::new(),
            item_name: String::new(),
        }
    }
}

impl StreamParser for DeriveStream {
    fn parse(tokens: TokenStream) -> Option<Self> {
        let mut parsed = DeriveStream::new();

        let mut iter = tokens.into_iter().peekable();

        if !iter
            .clone()
            .any(|tree| tree.to_string() == String::from("pub"))
        {
            Span::call_site()
                .error("the CommandMetadata trait can only be derived on pub items")
                .emit();
            return None;
        }

        if !iter
            .clone()
            .any(|tree| tree.to_string() == String::from("struct"))
        {
            Span::call_site()
                .error("the CommandMetadata trait can only be derived on structs")
                .emit();
            return None;
        }

        while let Some(first) = iter.next() && first.to_string() != String::from("pub") {
            // look for the beginning of an attribute
            //
            // #[metadata(name = "name")]
            // ^
            let TokenTree::Punct(punct) = first.clone() else {
                return None;
            };

            if punct.as_char() != '#' {
                first
                    .span()
                    .error("no metadata attributes found after derive")
                    .span_note(
                        Span::call_site(),
                        "metadata attributes are expected after the derive invocation",
                    )
                    .emit();
                return None;
            }

            // look for a token group of an attribute after the '#' symbol, delimited by square
            // brackets: []
            //
            // #[metadata(name = "name")]
            //  ^-----------------------^

            let TokenTree::Group(group) = iter.peek().unwrap() else {
                return None;
            };

            if group.delimiter() != Delimiter::Bracket {
                return None;
            }

            let mut group_tokens = group.stream().into_iter();
            let group_first = group_tokens.next().unwrap();

            // check if the attribute name is exactly equal to "metadata"
            //
            // #[metadata(name = "name")]
            //   --------
            let TokenTree::Ident(ident) = group_first.clone() else {
                return None;
            };

            if ident.to_string().as_str() != "metadata" {
                group_first
                    .span()
                    .error(format!(
                        "expected metadata attribute; found {group_first} attribute instead"
                    ))
                    .emit();
                return None;
            }

            let group_next_option = group_tokens.next();
            if group_next_option.is_none() {
                group_first
                    .span()
                    .error("unexpected end of attribute")
                    .emit();
                return None;
            }

            let group_next = group_next_option.unwrap();

            // look for a parenthesized group of parameter
            //
            // #[metadata(name = "name")]
            //           ^-------------^
            let TokenTree::Group(group) = group_next.clone() else {
                return None;
            };

            if group.delimiter() != Delimiter::Parenthesis {
                group_next
                    .span()
                    .error("expected parenthesized parameter")
                    .emit();
                return None;
            }

            if group.stream().is_empty() {
                group
                    .span()
                    .error("parameter expected; none found")
                    .note("valid parameters: description, name, type")
                    .emit();
                return None;
            }

            let mut group_inner_tokens = group.stream().into_iter().peekable();
            let first = group_inner_tokens.next().unwrap();

            // check if the parameter name is one of "description", "name" or "type"
            //
            // #[metadata(name = "name)]
            //            ----
            let TokenTree::Ident(ident) = first.clone() else {
                first
                    .span()
                    .error(format!("expected identifier; found {first} instead"))
                    .emit();
                return None;
            };

            let ident_string = ident.to_string();
            let ident_str = ident_string.as_str();
            if !VALID_ATTR_PARAMETER_NAMES.contains(&ident_str) {
                first
                    .span()
                    .error(format!("unexpected parameter name: {ident_string}"))
                    .note(format!("valid parameter names: {}", VALID_ATTR_PARAMETER_NAMES.join(", ")))
                    .emit();
                return None;
            }

            if group_inner_tokens.peek().is_none() {
                first
                    .span()
                    .error("unexpected end of parameter")
                    .emit();
                return None;
            }

            // check if the "=" sign follows the parameter name
            //
            // #[metadata(name = "name")]
            //                 ^
            let group_token_next = group_inner_tokens.next().unwrap();
            let TokenTree::Punct(punct) = group_token_next.clone() else {
                first
                    .span()
                    .error(format!("expected punctuation; found {group_token_next} instead"))
                    .emit();
                return None;
            };

            if punct.as_char() != '=' {
                punct
                    .span()
                    .error(format!("expected = punctuation; found {punct} punctuation instead"))
                    .emit();
                return None;
            }

            if group_inner_tokens.peek().is_none() {
                punct
                    .span()
                    .error("unexpected end of parameter")
                    .emit();
                return None;
            }

            let attribute = if LITERAL_PARAMETERS.contains(&ident_str) {
                // check if a literal follows the "=" sign (for parameters description, name and type)
                //
                // #[metadata(name = "name")]
                //                   ------
                // #[metadata(type = 1)]
                //                   -
                let group_token_next = group_inner_tokens.next().unwrap();
                let TokenTree::Literal(literal) = group_token_next.clone() else {
                    group_token_next
                        .span()
                        .error(format!("expected literal; found {group_token_next}"))
                        .emit();
                    return None;
                };

                match ident_str {
                    "description" => {
                        let description = literal.to_string();
                        DeriveAttribute::Description(description)
                    }
                    "name" => {
                        let name = literal.to_string();
                        DeriveAttribute::Name(name)
                    }
                    "type" => {
                        let Ok(int) = literal.to_string().parse::<u8>() else {
                            literal
                                .span()
                                .error(format!("expected integer literal; found literal {:?}", literal.to_string()))
                                .emit();
                            return None;
                        };

                        DeriveAttribute::Type(int)
                    }
                    _ => return None,
                }
            } else if BOOLEAN_PARAMETERS.contains(&ident_str) {
                // check if a boolean follows the "=" sign (for parameters interaction_only)
                //
                // #[metadata(interaction_only = true)]
                //                               ----
                let group_token_next = group_inner_tokens.next().unwrap();
                let TokenTree::Ident(ident) = group_token_next.clone() else {
                    group_token_next
                        .span()
                        .error(format!("expected identifier; found {group_token_next}"))
                        .emit();
                    return None;
                };

                match ident_str {
                    "interaction_only" => {
                        let Ok(boolean) = ident.to_string().parse::<bool>() else {
                            ident
                                .span()
                                .error(format!("expected boolean; found {ident}"))
                                .emit();
                            return None;
                        };

                        DeriveAttribute::InteractionOnly(boolean)
                    }
                    _ => return None,
                }
            } else {
                return None;
            };

            parsed.attributes.push(attribute);
            iter.next();
        }

        iter.next();
        parsed.item_name = iter.next().unwrap().to_string();

        Some(parsed)
    }
}
