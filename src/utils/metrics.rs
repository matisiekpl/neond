pub fn sanitize_metric_slug(slug: &str) -> String {
    slug.chars().map(|c| if c == '.' || c == '-' { '_' } else { c }).collect()
}