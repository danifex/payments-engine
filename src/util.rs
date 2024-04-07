use anyhow::Result;

pub fn fixed_point_4_decimal_to_float_str(value: u64) -> String {
    format!("{}.{:04}", value / 10_000, value % 10_000)
}

pub fn signed_fixed_point_4_decimal_to_float_str(value: i64) -> String {
    format!(
        "{}{}",
        get_sign_prefix(value),
        fixed_point_4_decimal_to_float_str(value.unsigned_abs())
    )
}

fn get_sign_prefix(value: i64) -> &'static str {
    if value < 0 {
        "-"
    } else {
        ""
    }
}

pub fn float_str_to_fixed_point_4_decimal(value: &str) -> Result<u64> {
    let (integer, fractional) = match value.split_once('.') {
        None => (value, ""),
        Some((p, s)) => (p, s),
    };

    let integer = integer.parse::<u64>()? * 10_000;
    let fractional = first_four_chars_or_pad(fractional).parse::<u64>()?;

    Ok(integer + fractional)
}

fn first_four_chars_or_pad(s: &str) -> String {
    let mut result = s.to_string();
    while result.len() < 4 {
        result.push('0');
    }
    result[..4].to_string()
}

#[cfg(test)]
mod tests {
    use crate::util::{
        fixed_point_4_decimal_to_float_str, float_str_to_fixed_point_4_decimal,
        signed_fixed_point_4_decimal_to_float_str,
    };

    #[test]
    fn test_fixed_4_decimal_points_to_float() {
        assert_eq!(fixed_point_4_decimal_to_float_str(0), "0.0000");
        assert_eq!(fixed_point_4_decimal_to_float_str(1), "0.0001");
        assert_eq!(fixed_point_4_decimal_to_float_str(9_999), "0.9999");
        assert_eq!(fixed_point_4_decimal_to_float_str(10_000), "1.0000");
        assert_eq!(fixed_point_4_decimal_to_float_str(10_001), "1.0001");
    }

    #[test]
    fn test_signed_fixed_4_decimal_points_to_float() {
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(0), "0.0000");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(1), "0.0001");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(9_999), "0.9999");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(10_000), "1.0000");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(10_001), "1.0001");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(-0), "0.0000");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(-1), "-0.0001");
        assert_eq!(signed_fixed_point_4_decimal_to_float_str(-9_999), "-0.9999");
        assert_eq!(
            signed_fixed_point_4_decimal_to_float_str(-10_000),
            "-1.0000"
        );
        assert_eq!(
            signed_fixed_point_4_decimal_to_float_str(-10_001),
            "-1.0001"
        );
    }

    #[test]
    fn test_float_str_to_fixed_point_4_decimal() {
        assert_eq!(float_str_to_fixed_point_4_decimal("0").unwrap(), 0);
        assert_eq!(float_str_to_fixed_point_4_decimal("0.0001").unwrap(), 1);
        assert_eq!(float_str_to_fixed_point_4_decimal("0.9999").unwrap(), 9_999);
        assert_eq!(
            float_str_to_fixed_point_4_decimal("1.0000").unwrap(),
            10_000
        );
        assert_eq!(
            float_str_to_fixed_point_4_decimal("1.0001").unwrap(),
            10_001
        );

        // Test extra digits in fractional part
        assert_eq!(
            float_str_to_fixed_point_4_decimal("1.00019999").unwrap(),
            10_001
        );
        assert_eq!(
            float_str_to_fixed_point_4_decimal("1.00010000").unwrap(),
            10_001
        );

        // Test correct padding
        assert_eq!(float_str_to_fixed_point_4_decimal("0.99").unwrap(), 9_900);
        assert_eq!(float_str_to_fixed_point_4_decimal("0.990").unwrap(), 9_900);
    }
}
