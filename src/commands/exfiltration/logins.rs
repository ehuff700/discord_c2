use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Login {
	pub(crate) origin_url:       String,
	pub(crate) action_url:       String,
	pub(crate) username_value:   String,
	pub(crate) password_value:   String,
	pub(crate) created_at:       String,
	pub(crate) last_login_at:    String,
	pub(crate) last_modified_at: String,
}

impl fmt::Display for Login {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let created_at = &self.created_at;
		let last_login_at = &self.last_login_at;
		let last_modified_at = &self.last_modified_at;
		let password = self.password_value.to_string();

		write!(
			f,
			"Account(\n\tOrigin URL: {}\n\tAction URL: {}\n\tUsername: {}\n\tPassword: {}\n\tCreated at: {}\n\tLast login at: {}\n\tLast modified at: {}\n)",
			self.origin_url, self.action_url, self.username_value, password, created_at, last_login_at, last_modified_at
		)
	}
}
