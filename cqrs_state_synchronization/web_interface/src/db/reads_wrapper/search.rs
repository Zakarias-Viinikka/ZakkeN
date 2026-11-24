use client_table_blueprints::tbl_every_block_in_existence::{
    self, CONTENT, ID_OF_PAGE_I_BELONG_TO, IS_TITLE, MY_ID_AS_GIVEN_BY_YRS, PAGE_IS_DISABLED,
};
use protocol::error::DbError;
use protocol::payload::{
    GetDataIn, JoinType, SearchFts5In, SelectArgument, SelectArguments,
};
use web_internal_db::db_helper;

use super::destruct::{destruct_entire_get_all_menu_pages, destruct_id};
use crate::complex_state::LocalPages;

pub async fn search_page_titles(text_to_lookup: &str) -> Result<Vec<LocalPages>, DbError> {


    // FTS5 trigram MATCH requires at least three characters per search term.
    // Shorter terms use a normal read and Rust substring matching.
    if text_to_lookup.trim().is_empty()
        || text_to_lookup
            .split_whitespace()
            .any(|word| word.chars().count() < 3)
    {
        let out = db_helper::get_data(GetDataIn {
            table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
            arguments: SelectArguments::Two {
                first: SelectArgument::XEqualY {
                    x: IS_TITLE.name.to_string(),
                    y: "true".to_string(),
                },
                join: JoinType::And,
                second: SelectArgument::XEqualY {
                    x: PAGE_IS_DISABLED.name.to_string(),
                    y: "false".to_string(),
                },
            },
            columns_to_read: vec![
                "id".to_string(),
                CONTENT.name.to_string(),
                MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
                ID_OF_PAGE_I_BELONG_TO.name.to_string(),
            ],
        })
        .await?;

        let mut result = Vec::new();
        for row in out.rows {
            let page = destruct_entire_get_all_menu_pages(row)?;
            if title_matches_query(&page.title, text_to_lookup) {
                result.push(page);
            }
        }
        return Ok(result);
    }

    let search_out = db_helper::search_fts5(SearchFts5In {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        text_to_lookup: text_to_lookup.to_string(),
    })
    .await?;

    let mut matching_ids = Vec::new();
    for row in search_out.rows {
        let rowid_col = row
            .cols
            .into_iter()
            .next()
            .ok_or_else(|| DbError::ColDestructFail("search row missing rowid".into()))?;
        matching_ids.push(destruct_id(rowid_col)?.to_string());
    }

    if matching_ids.is_empty() {
        return Ok(Vec::new());
    }

    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Three {
            first: SelectArgument::XInY {
                x: "id".to_string(),
                y: matching_ids,
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: IS_TITLE.name.to_string(),
                y: "true".to_string(),
            },
            join2: JoinType::And,
            third: SelectArgument::XEqualY {
                x: PAGE_IS_DISABLED.name.to_string(),
                y: "false".to_string(),
            },
        },
        columns_to_read: vec![
            "id".to_string(),
            CONTENT.name.to_string(),
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
            ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        ],
    })
    .await?;

    let mut result = Vec::new();
    for row in out.rows {
        result.push(destruct_entire_get_all_menu_pages(row)?);
    }
    Ok(result)
}

fn title_matches_query(title: &str, query: &str) -> bool {
    let lowercase_title = title.to_lowercase();

    query
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .all(|word| lowercase_title.contains(word.as_str()))
}

