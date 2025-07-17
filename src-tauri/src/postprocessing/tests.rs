#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_works() {
        let raw = "```\nhello\n```";
        assert_eq!(clean(raw), "hello");
    }

    #[test]
    fn persona_applies() {
        let filter = persona::PersonaApplier {
            name: "Erika".into(),
        };
        assert_eq!(filter.apply("hi"), "*giggles* hi");
    }

    #[test]
    fn validate_safe() {
        assert!(validate("safe text").is_ok());
    }

    #[test]
    fn validate_rejects_script() {
        assert!(validate("<script>alert()</script>").is_err());
    }
}