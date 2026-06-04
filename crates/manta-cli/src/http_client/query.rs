//! Chainable query-string builder used by the per-resource HTTP
//! methods to assemble the `&[(&str, String)]` slice that
//! `MantaClient::get_json` (and friends) expect.

/// Chainable builder for the `&[(&str, String)]` query-pairs slice
/// that `MantaClient::get_json` expects. Each `.opt()` / `.vec()` /
/// `.flag()` / `.pair()` call mirrors one of the patterns the older
/// hand-written query blocks used; absent values are skipped.
#[derive(Default)]
pub struct QueryBuilder {
  pairs: Vec<(&'static str, String)>,
}

impl QueryBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  /// Push `(name, value.clone())` only when `value` is `Some`.
  pub fn opt(
    mut self,
    name: &'static str,
    value: &Option<String>,
  ) -> Self {
    if let Some(v) = value {
      self.pairs.push((name, v.clone()));
    }
    self
  }

  /// Push `(name, value.to_string())` only when `value` is `Some`.
  /// For numeric `Option<T>` where `T: ToString`.
  pub fn opt_display<T: ToString>(
    mut self,
    name: &'static str,
    value: &Option<T>,
  ) -> Self {
    if let Some(v) = value {
      self.pairs.push((name, v.to_string()));
    }
    self
  }

  /// Push `(name, items.join(","))` only when `items` is non-empty.
  pub fn vec(mut self, name: &'static str, items: &[String]) -> Self {
    if !items.is_empty() {
      self.pairs.push((name, items.join(",")));
    }
    self
  }

  /// Push `(name, "true")` only when `value` is `true`.
  pub fn flag(mut self, name: &'static str, value: bool) -> Self {
    if value {
      self.pairs.push((name, "true".to_string()));
    }
    self
  }

  /// Push `(name, value)` unconditionally.
  pub fn pair(mut self, name: &'static str, value: String) -> Self {
    self.pairs.push((name, value));
    self
  }

  /// Consume into the slice-shaped form `get_json` accepts.
  pub fn build(self) -> Vec<(&'static str, String)> {
    self.pairs
  }
}
