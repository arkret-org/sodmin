//! Typed Realm organization control API client.

use std::collections::BTreeSet;

use arkret_models_collaboration::governance::realm_governance::RealmOrganizationRelationshipList;
use coauth_admin_types::organization_admin::OrganizationControlView;

use crate::api::client::{NO_BODY, api_client};
use crate::utils::net::error::HttpError;

#[derive(Clone, Debug)]
pub struct RealmOrganizationAdminView {
    pub relationships: RealmOrganizationRelationshipList,
    pub controls: Vec<OrganizationControlView>,
    /// Organizations that are valid relationship/hint rows but are not
    /// governed by this deployment's Account Authority.
    pub unavailable_control_dids: Vec<String>,
}

pub async fn get_realm_organization_admin_view(
    realm_id: &str,
) -> Result<RealmOrganizationAdminView, HttpError> {
    let relationship_path = format!(
        "/_soland/admin/realms/{}/organizations",
        urlencoding::encode(realm_id)
    );
    let relationships: RealmOrganizationRelationshipList =
        api_client(&relationship_path, "GET", NO_BODY).await?;
    if relationships.realm_id.to_string() != realm_id {
        return Err(HttpError::message(
            "realm organization response did not match the requested realm",
        ));
    }

    let organization_dids: BTreeSet<String> = relationships
        .relationships
        .iter()
        .map(|row| row.organization_id.to_string())
        .chain(
            relationships
                .declared_organization_hints
                .iter()
                .map(ToString::to_string),
        )
        .collect();

    let mut controls = Vec::new();
    let mut unavailable_control_dids = Vec::new();
    for organization_did in organization_dids {
        let path = format!(
            "/_coauth/admin/organizations/{}",
            urlencoding::encode(&organization_did)
        );
        match api_client::<OrganizationControlView, _>(&path, "GET", NO_BODY).await {
            Ok(view) => controls.push(view),
            Err(error) if error.status == 404 => unavailable_control_dids.push(organization_did),
            Err(error) => return Err(error),
        }
    }

    Ok(RealmOrganizationAdminView {
        relationships,
        controls,
        unavailable_control_dids,
    })
}
