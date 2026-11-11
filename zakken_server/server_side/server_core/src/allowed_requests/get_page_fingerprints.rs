use db_core::db::{db_actions_for_drift, operations_pages};
use db_wrapper::android_mascot::LiveForever;
use drift_guard_shared::fingerprint;
use protocol::serialization::Convert;
use server_errors::{ErrorDetails, ServErr};
use server_protocol::{GetPageFingerprintsPayload, PageFingerprint};

/// Return page_id + fingerprint for every page the server has, plus the
/// current drift version. Exempt from the drift check — a lonewolf
/// client needs this to compare. Fingerprints the raw blob bytes: a
/// content-identical page may still mismatch, which just means an
/// extra card on the client. That's cheaper than decoding every page.
pub fn get_page_fingerprints(_msg_content: Vec<u8>, db_wrapper: &LiveForever) -> Vec<u8> {
    let result: Result<GetPageFingerprintsPayload, ServErr> = (|| {
        let drift_version = current_drift_version(db_wrapper)?;
        let pages = fingerprint_every_page(db_wrapper)?;
        Ok(GetPageFingerprintsPayload {
            drift_version,
            pages,
        })
    })();
    result.to_payload()
}

fn current_drift_version(db_wrapper: &LiveForever) -> Result<String, ServErr> {
    match db_actions_for_drift::get_current(db_wrapper)? {
        Some(dv) => Ok(dv.version),
        None => Err(ServErr::ServerError(ErrorDetails {
            err_msg: "server has no drift_version row".to_string(),
            file: file!().to_string(),
            method: "get_page_fingerprints::current_drift_version".to_string(),
            probable_cause: "ensure_minted didn't run at startup".to_string(),
        })),
    }
}

fn fingerprint_every_page(db_wrapper: &LiveForever) -> Result<Vec<PageFingerprint>, ServErr> {
    let stored = operations_pages::get_all_pages(db_wrapper)?;
    Ok(stored
        .into_iter()
        .map(|p| PageFingerprint {
            page_id: p.page_id,
            fingerprint: fingerprint(p.blobbed_page.as_slice()),
        })
        .collect())
}
