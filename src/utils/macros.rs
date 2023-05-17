/// Formats and styles the given label and value with ANSI colors.
///
/// This macro takes four arguments: `label_color`, `value_color`, `label`, and `value`.
/// It formats and styles the `label` and `value` using the specified ANSI colors and returns the result as a `String`.
///
/// # Arguments
///
/// - `$label_color`: The ANSI color style to apply to the `label`.
/// - `$value_color`: The ANSI color style to apply to the `value`.
/// - `$label`: The label to format and style.
/// - `$value`: The value to format and style.
///
/// # Example
///
/// ```
/// use ansi_term::Color;
///
/// let label_color = Color.Cyan();
/// let value_color = Color::Yellow();
/// let label = "Name:";
/// let value = "John Doe";
///
/// let formatted = formatted_ansi!(label_color, value_color, label, value);
/// println!("{}", formatted);
/// ```
///
/// This will print the formatted string where the `label` is colored blue and the `value` is colored yellow and bold.
///
/// # Note
///
/// This macro requires the `ansi_term` crate to be imported and used for ANSI color support.
#[macro_export]
macro_rules! formatted_ansi {
	($label_color:expr, $value_color:expr, $label:expr, $value:expr) => {
		format!("{}{}", $label_color.paint($label), $value_color.paint($value),)
	};
}
