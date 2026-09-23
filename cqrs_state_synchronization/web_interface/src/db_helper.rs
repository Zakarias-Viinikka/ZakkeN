use js_sys::Array;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::serialization::Convert;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    fn javascript_im_begging_you(args: &JsValue) -> js_sys::Promise;
}

async fn tell_worker_to_do(cmd: &str, payload: Option<Vec<u8>>) -> Result<Vec<u8>, DbError> {
    let arr = Array::new();
    arr.push(&JsValue::from_str(cmd));

    if let Some(bytes) = payload {
        arr.push(&js_sys::Uint8Array::from(&bytes[..]));
    }

    let promise = javascript_im_begging_you(&arr);
    let raw = JsFuture::from(promise)
        .await
        .map_err(|e| DbError::CureFail(format!("promise rejected: {:?}", e)))?;
    let result_in_js_form: Array = raw
        .dyn_into()
        .map_err(|_| DbError::CureFail("response was not an array".into()))?;
    let js_form_of_rust_vecu8: js_sys::Uint8Array = result_in_js_form
        .get(1)
        .dyn_into()
        .map_err(|_| DbError::CureFail("response[1] was not a Uint8Array".into()))?;
    let mut rust_vecu8_with_same_length_as_js_version_of_vecu8_but_every_u8_is_zero =
        vec![0u8; js_form_of_rust_vecu8.length() as usize];
    js_form_of_rust_vecu8
        .copy_to(&mut rust_vecu8_with_same_length_as_js_version_of_vecu8_but_every_u8_is_zero);
    let vecu8_final = rust_vecu8_with_same_length_as_js_version_of_vecu8_but_every_u8_is_zero;
    Ok(vecu8_final)
}

// no input, output
pub async fn list_tables() -> Result<ListTablesOut, DbError> {
    let bytes = tell_worker_to_do("list_tables", None).await?;
    ListTablesOut::un_payloadify(&bytes)
}

pub async fn export_database() -> Result<ExportDatabaseOut, DbError> {
    let bytes = tell_worker_to_do("export_database", None).await?;
    ExportDatabaseOut::un_payloadify(&bytes)
}

// no input, no output
pub async fn begin_all_or_nothing() -> Result<(), DbError> {
    let bytes = tell_worker_to_do("begin_all_or_nothing", None).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn everything_went_perfectly() -> Result<(), DbError> {
    let bytes = tell_worker_to_do("everything_went_perfectly", None).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn regret_everything() -> Result<(), DbError> {
    let bytes = tell_worker_to_do("regret_everything", None).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

// input, no output
pub async fn create_table(input: CreateTableIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("create_table", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn drop_table(input: DropTableIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("drop_table", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn insert_data(input: InsertDataIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("insert_data", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn delete_row(input: DeleteRowIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("delete_row", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn edit_col_in_row(input: EditColInRowIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("edit_col_in_row", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn edit_col_in_row_where(input: EditColInRowWhereIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("edit_col_in_row_where", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn swap_columns(input: SwapColumnsIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("swap_columns", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn create_index(input: CreateIndexIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("create_index", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn add_column(input: AddColumnIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("add_column", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn remove_column(input: RemoveColumnIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("remove_column", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn fundamentally_edit_existing_col(
    input: FundamentallyEditExistingColIn,
) -> Result<(), DbError> {
    let bytes =
        tell_worker_to_do("fundamentally_edit_existing_col", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn create_table_from_export(input: CreateTableFromExportIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("create_table_from_export", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn copy_table(input: CopyTableIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("copy_table", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn create_foreign_table(input: CreateForeignTableIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("create_foreign_table", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn create_fts5_table(input: CreateFts5TableIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("create_fts5_table", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn rebuild_fts5_index(input: RebuildFts5In) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("rebuild_fts5_index", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

pub async fn force_drop_table(input: DropTableIn) -> Result<(), DbError> {
    let bytes = tell_worker_to_do("force_drop_table", Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

// input, output
pub async fn get_data(input: GetDataIn) -> Result<GetDataOut, DbError> {
    let bytes = tell_worker_to_do("get_data", Some(input.to_payload())).await?;
    GetDataOut::un_payloadify(&bytes)
}

pub async fn get_data_ordered(input: GetDataOrderedIn) -> Result<GetDataOut, DbError> {
    let bytes = tell_worker_to_do("get_data_ordered", Some(input.to_payload())).await?;
    GetDataOut::un_payloadify(&bytes)
}

pub async fn check_table(input: CheckTableIn) -> Result<CheckTableOut, DbError> {
    let bytes = tell_worker_to_do("check_table", Some(input.to_payload())).await?;
    CheckTableOut::un_payloadify(&bytes)
}

pub async fn check_index(input: CheckIndexIn) -> Result<CheckIndexOut, DbError> {
    let bytes = tell_worker_to_do("check_index", Some(input.to_payload())).await?;
    CheckIndexOut::un_payloadify(&bytes)
}

pub async fn export_tables(input: ExportTablesIn) -> Result<ExportTablesOut, DbError> {
    let bytes = tell_worker_to_do("export_tables", Some(input.to_payload())).await?;
    ExportTablesOut::un_payloadify(&bytes)
}

pub async fn count_all_rows(input: CountAllRowsIn) -> Result<CountAllRowsOut, DbError> {
    let bytes = tell_worker_to_do("count_all_rows", Some(input.to_payload())).await?;
    CountAllRowsOut::un_payloadify(&bytes)
}

pub async fn search_fts5(input: SearchFts5In) -> Result<SearchFts5Out, DbError> {
    let bytes = tell_worker_to_do("search_fts5", Some(input.to_payload())).await?;
    SearchFts5Out::un_payloadify(&bytes)
}
