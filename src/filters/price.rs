pub fn format_price_filter(value: f64, decimal_places: Option<usize>) -> String {
    format!("{value:.0$}", decimal_places.unwrap_or(2))
}
