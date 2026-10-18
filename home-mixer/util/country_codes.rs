const TOP_COUNTRIES: &[&str] = &[
    "JP", "US", "TR", "SA", "BR", "GB", "ID", "IN", "MX", "TH", "KR", "FR", "NG", "CA", "ES", "AR",
    "PH", "DE",
];

pub fn bucket_country(country_code: &str) -> &str {
    if TOP_COUNTRIES.contains(&country_code) {
        country_code
    } else {
        "other"
    }
}
