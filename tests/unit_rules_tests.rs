#[cfg(test)]
mod unit_rules_tests {
    use cmop::{expressions::*, rules::*};

    #[test]
    fn test_notes() {
        let mods: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
            .iter()
            .map(|e| (*e).into())
            .collect();

        let rules: Vec<_> = [("a", "some a"), ("c", "some b"), ("x", "some x!")]
            .iter()
            .map(|e| Note {
                comment: e.1.into(),
                expression: Atomic { item: e.0.into() }.into(),
            })
            .collect();

        let mut warnings: Vec<String> = vec![];
        for rule in rules {
            if rule.eval(&mods) {
                warnings.push(rule.get_comment().into());
            }
        }
        let expected: Vec<String> = vec!["some a".to_owned(), "some b".into()];
        assert_eq!(warnings, expected);
    }

    #[test]
    fn test_conflicts() {
        let mods: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
            .iter()
            .map(|e| (*e).into())
            .collect();

        let rule1 = Conflict {
            comment: "a conflicts with b".into(),
            expression_a: Atomic { item: "a".into() }.into(),
            expression_b: Atomic { item: "b".into() }.into(),
        };
        let rule2 = Conflict {
            comment: "b conflicts with x".into(),
            expression_a: Atomic { item: "b".into() }.into(),
            expression_b: Atomic { item: "x".into() }.into(),
        };
        let rules: Vec<Conflict> = vec![
            rule1, // a conflicts with a
            rule2, // b conflicts with x
        ];

        let mut warnings: Vec<String> = vec![];
        for rule in rules {
            if rule.eval(&mods) {
                warnings.push(rule.get_comment().into());
            }
        }
        let expected: Vec<String> = vec!["a conflicts with b".into()];
        assert_eq!(warnings, expected);
    }

    #[test]
    fn test_requires() {
        let mods: Vec<String> = ["a", "b", "c", "d", "e", "f", "g"]
            .iter()
            .map(|e| (*e).into())
            .collect();

        let rules: Vec<Require> = vec![
            // a requires b
            Require {
                comment: "a requires b".into(),
                expression_a: Atomic { item: "a".into() }.into(),
                expression_b: Atomic { item: "b".into() }.into(),
            },
            // b requires x
            Require {
                comment: "b requires x".into(),
                expression_a: Atomic { item: "b".into() }.into(),
                expression_b: Atomic { item: "x".into() }.into(),
            },
            // x requires y
            Require {
                comment: "x requires y".into(),
                expression_a: Atomic { item: "x".into() }.into(),
                expression_b: Atomic { item: "y".into() }.into(),
            },
        ];

        let mut warnings: Vec<String> = vec![];
        for rule in rules {
            if rule.eval(&mods) {
                warnings.push(rule.get_comment().into());
            }
        }
        let expected: Vec<String> = vec!["b requires x".to_owned()];
        assert_eq!(warnings, expected);
    }
}
