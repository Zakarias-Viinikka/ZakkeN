use client_table_blueprints::tbl_pages;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{
    ColumnValue, CreateTableIn, EditColInRowWhereIn, GetDataIn, InsertDataIn, SelectArgument,
    SelectArguments,
};
use protocol::row_col::Col;
use protocol::schema_helper::DestructDbReturnCol;

pub struct StoredPage {
    pub page_id: String,
    pub blobbed_page: Vec<u8>,
    pub page_status: Vec<u8>,
    pub version: Vec<u8>,
    pub is_main_menu_page: String,
}

pub fn create_table_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;
    if list
        .table_names
        .iter()
        .any(|t| t == tbl_pages::TABLE_NAME)
    {
        return Ok(());
    }

    liver.create_table(CreateTableIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        columns: tbl_pages::new_table_pages(),
    })
}

pub fn page_exists(liver: &LiveForever, page_id: &str) -> Result<bool, DbError> {
    let out = liver.get_data(GetDataIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id.to_string(),
        }),
        columns_to_read: vec![tbl_pages::PAGE_ID.name.to_string()],
    })?;
    Ok(!out.rows.is_empty())
}

pub fn get_page(liver: &LiveForever, page_id: &str) -> Result<Option<StoredPage>, DbError> {
    let out = liver.get_data(GetDataIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id.to_string(),
        }),
        columns_to_read: vec![
            tbl_pages::PAGE_ID.name.to_string(),
            tbl_pages::BLOBBED_PAGE.name.to_string(),
            tbl_pages::PAGE_STATUS.name.to_string(),
            tbl_pages::VERSION.name.to_string(),
            tbl_pages::IS_MAIN_MENU_PAGE.name.to_string(),
        ],
    })?;

    let Some(row) = out.rows.into_iter().next() else {
        return Ok(None);
    };

    let mut cols = row.cols.into_iter();
    let page_id_col = cols
        .next()
        .ok_or_else(|| DbError::BadCode("pages row missing page_id".to_string()))?;
    let blobbed_page_col = cols
        .next()
        .ok_or_else(|| DbError::BadCode("pages row missing blobbed_page".to_string()))?;
    let page_status_col = cols
        .next()
        .ok_or_else(|| DbError::BadCode("pages row missing page_status".to_string()))?;
    let version_col = cols
        .next()
        .ok_or_else(|| DbError::BadCode("pages row missing version".to_string()))?;
    let is_main_menu_page_col = cols
        .next()
        .ok_or_else(|| DbError::BadCode("pages row missing is_main_menu_page".to_string()))?;

    let page_id = tbl_pages::PAGE_ID
        .try_destruct_db_col(page_id_col)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("page_id was null".to_string()))?;
    let blobbed_page = tbl_pages::BLOBBED_PAGE
        .try_destruct_db_col(blobbed_page_col)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("blobbed_page was null".to_string()))?;
    let page_status = tbl_pages::PAGE_STATUS
        .try_destruct_db_col(page_status_col)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("page_status was null".to_string()))?;
    let version = tbl_pages::VERSION
        .try_destruct_db_col(version_col)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("version was null".to_string()))?;
    let is_main_menu_page = tbl_pages::IS_MAIN_MENU_PAGE
        .try_destruct_db_col(is_main_menu_page_col)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("is_main_menu_page was null".to_string()))?;

    Ok(Some(StoredPage {
        page_id,
        blobbed_page,
        page_status,
        version,
        is_main_menu_page,
    }))
}

pub fn get_all_pages(liver: &LiveForever) -> Result<Vec<StoredPage>, DbError> {
    let out = liver.get_data(GetDataIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![
            tbl_pages::PAGE_ID.name.to_string(),
            tbl_pages::BLOBBED_PAGE.name.to_string(),
            tbl_pages::PAGE_STATUS.name.to_string(),
            tbl_pages::VERSION.name.to_string(),
            tbl_pages::IS_MAIN_MENU_PAGE.name.to_string(),
        ],
    })?;

    let mut pages = Vec::new();
    for row in out.rows {
        let mut cols = row.cols.into_iter();
        let page_id_col = cols
            .next()
            .ok_or_else(|| DbError::BadCode("pages row missing page_id".to_string()))?;
        let blobbed_page_col = cols
            .next()
            .ok_or_else(|| DbError::BadCode("pages row missing blobbed_page".to_string()))?;
        let page_status_col = cols
            .next()
            .ok_or_else(|| DbError::BadCode("pages row missing page_status".to_string()))?;
        let version_col = cols
            .next()
            .ok_or_else(|| DbError::BadCode("pages row missing version".to_string()))?;
        let is_main_menu_page_col = cols
            .next()
            .ok_or_else(|| DbError::BadCode("pages row missing is_main_menu_page".to_string()))?;

        let page_id = tbl_pages::PAGE_ID
            .try_destruct_db_col(page_id_col)
            .map_err(DbError::BadCode)?
            .ok_or_else(|| DbError::BadCode("page_id was null".to_string()))?;
        let blobbed_page = tbl_pages::BLOBBED_PAGE
            .try_destruct_db_col(blobbed_page_col)
            .map_err(DbError::BadCode)?
            .ok_or_else(|| DbError::BadCode("blobbed_page was null".to_string()))?;
        let page_status = tbl_pages::PAGE_STATUS
            .try_destruct_db_col(page_status_col)
            .map_err(DbError::BadCode)?
            .ok_or_else(|| DbError::BadCode("page_status was null".to_string()))?;
        let version = tbl_pages::VERSION
            .try_destruct_db_col(version_col)
            .map_err(DbError::BadCode)?
            .ok_or_else(|| DbError::BadCode("version was null".to_string()))?;
        let is_main_menu_page = tbl_pages::IS_MAIN_MENU_PAGE
            .try_destruct_db_col(is_main_menu_page_col)
            .map_err(DbError::BadCode)?
            .ok_or_else(|| DbError::BadCode("is_main_menu_page was null".to_string()))?;

        pages.push(StoredPage {
            page_id,
            blobbed_page,
            page_status,
            version,
            is_main_menu_page,
        });
    }

    Ok(pages)
}

pub fn insert_page(
    liver: &LiveForever,
    page_id: &str,
    blobbed_page: Vec<u8>,
    page_status: Vec<u8>,
    version: Vec<u8>,
    is_main_menu_page: &str,
) -> Result<(), DbError> {
    liver.insert_data(InsertDataIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        values: vec![
            ColumnValue {
                column_name: tbl_pages::PAGE_ID.name.to_string(),
                value: Col::Text(page_id.to_string()),
            },
            ColumnValue {
                column_name: tbl_pages::BLOBBED_PAGE.name.to_string(),
                value: Col::Blob(blobbed_page),
            },
            ColumnValue {
                column_name: tbl_pages::PAGE_STATUS.name.to_string(),
                value: Col::Blob(page_status),
            },
            ColumnValue {
                column_name: tbl_pages::VERSION.name.to_string(),
                value: Col::Blob(version),
            },
            ColumnValue {
                column_name: tbl_pages::IS_MAIN_MENU_PAGE.name.to_string(),
                value: Col::Text(is_main_menu_page.to_string()),
            },
        ],
    })
}

pub fn update_page_blob(
    liver: &LiveForever,
    page_id: &str,
    blobbed_page: Vec<u8>,
) -> Result<(), DbError> {
    liver.edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id.to_string(),
        }),
        column: tbl_pages::BLOBBED_PAGE.name.to_string(),
        new_value: Col::Blob(blobbed_page),
    })
}

pub fn update_page_status(
    liver: &LiveForever,
    page_id: &str,
    page_status: Vec<u8>,
) -> Result<(), DbError> {
    liver.edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id.to_string(),
        }),
        column: tbl_pages::PAGE_STATUS.name.to_string(),
        new_value: Col::Blob(page_status),
    })
}