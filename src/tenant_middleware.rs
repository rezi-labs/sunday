use actix_web::HttpRequest;

/// Extract tenant name from URL path
/// Expected format: /{tenant}/...
pub fn get_tenant_from_path(req: &HttpRequest) -> Option<String> {
    let path = req.path();
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    if path_segments.is_empty() {
        return None;
    }

    let tenant_name = path_segments[0];

    // Validate tenant name format (alphanumeric and underscores only)
    if tenant_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(tenant_name.to_string())
    } else {
        None
    }
}
