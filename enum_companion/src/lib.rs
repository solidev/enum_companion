#![doc = include_str!("../README.md")]

pub use enum_companion_derive::EnumCompanion;
/// A trait for accessing and updating struct fields dynamically.
///
/// This trait is automatically implemented for structs that derive `EnumCompanion`
/// and use the default method names.
pub trait EnumCompanionTrait<F, V>
where
    F: Copy + 'static,
{
    /// Returns the value of a specific field.
    fn value(&self, field: F) -> V;

    /// Updates the value of a specific field.
    fn update(&mut self, value: V);

    /// Returns an array of all field enum variants.
    fn fields() -> &'static [F];

    /// Returns a vector of all field values.
    fn as_values(&self) -> Vec<V>;
}

extern crate self as enum_companion;

// Tests
#[cfg(test)]
mod tests {
    use enum_companion_derive::EnumCompanion;

    #[test]
    fn test_simple_enum_companion() {
        // This test is just used to get the macro output for the README
        #[allow(dead_code)]
        #[derive(EnumCompanion)]
        #[companion(derive_value(Debug, PartialEq))]
        struct Example {
            id: u32,
            name: String,
        }
    }

    #[test]
    fn test_enum_companion() {
        #[derive(EnumCompanion)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        struct Test {
            name: String,
            distance: u32,
        }

        let test = Test {
            name: "Test".to_string(),
            distance: 42,
        };
        assert_eq!(
            test.value(TestField::Name),
            TestValue::Name("Test".to_string())
        );
        assert_eq!(Test::fields(), [TestField::Name, TestField::Distance]);
    }

    #[test]
    fn test_with_lifetime() {
        #[derive(EnumCompanion)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        struct TestLifetime<'a> {
            name: &'a str,
            distance: u32,
        }

        let test = TestLifetime {
            name: "Test",
            distance: 42,
        };
        assert_eq!(
            test.value(TestLifetimeField::Name),
            TestLifetimeValue::Name("Test")
        );
        assert_eq!(
            TestLifetime::fields(),
            [TestLifetimeField::Name, TestLifetimeField::Distance]
        );
    }

    #[test]
    fn test_with_generic() {
        #[derive(EnumCompanion)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        struct TestGeneric<T: Clone + PartialEq + std::fmt::Debug> {
            name: String,
            data: T,
        }

        let test = TestGeneric {
            name: "Test".to_string(),
            data: 42u32,
        };
        assert_eq!(
            test.value(TestGenericField::Data),
            TestGenericValue::Data(42u32)
        );

        let test2 = TestGeneric {
            name: "Test2".to_string(),
            data: "hello".to_string(),
        };
        assert_eq!(
            test2.value(TestGenericField::Data),
            TestGenericValue::Data("hello".to_string())
        );
        assert_eq!(
            TestGeneric::<String>::fields(),
            [TestGenericField::Name, TestGenericField::Data]
        );
    }

    mod nested {
        use super::*;

        #[derive(EnumCompanion)]
        #[allow(dead_code)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        pub(super) struct TestVisibility {
            pub name: String,
        }
    }

    #[test]
    fn test_visibility() {
        let test = nested::TestVisibility {
            name: "Test".to_string(),
        };
        assert_eq!(
            test.value(nested::TestVisibilityField::Name),
            nested::TestVisibilityValue::Name("Test".to_string())
        );
        assert_eq!(
            nested::TestVisibility::fields(),
            &[nested::TestVisibilityField::Name]
        );
    }

    #[test]
    fn test_from_str() {
        #[allow(dead_code)]
        #[derive(EnumCompanion)]
        #[companion(derive_field(PartialEq, Debug))]
        struct Test {
            field_one: String,
            #[companion(rename = "Field2")]
            field_two: u32,
        }

        use std::str::FromStr;
        assert_eq!(TestField::from_str("field_one"), Ok(TestField::FieldOne));
        assert_eq!(TestField::from_str("FieldOne"), Ok(TestField::FieldOne));
        assert_eq!(TestField::from_str("field_two"), Ok(TestField::Field2));
        assert_eq!(TestField::from_str("Field2"), Ok(TestField::Field2));
        assert!(TestField::from_str("field_three").is_err());
    }

    #[test]
    fn test_trait() {
        #[derive(EnumCompanion)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        struct Test {
            name: String,
            distance: u32,
        }

        let mut test = Test {
            name: "Test".to_string(),
            distance: 42,
        };

        assert_eq!(
            test.value(TestField::Name),
            TestValue::Name("Test".to_string())
        );
        test.update(TestValue::Distance(100));
        assert_eq!(test.distance, 100);
        assert_eq!(Test::fields(), &[TestField::Name, TestField::Distance]);
        assert_eq!(
            test.as_values(),
            vec![
                TestValue::Name("Test".to_string()),
                TestValue::Distance(100)
            ]
        );
    }

    #[test]
    fn test_with_serde() {
        use serde::{Deserialize, Serialize};
        #[derive(EnumCompanion)]
        #[companion(
            value_fn = "get_field",
            update_fn = "set_field",
            fields_fn = "get_all_fields",
            derive_field(Hash, Eq, PartialEq, Debug, Serialize, Deserialize),
            derive_value(Serialize, Deserialize, Debug, PartialEq),
            serde_field(rename_all = "camelCase"),
            serde_value(rename_all = "camelCase", tag = "type", content = "value")
        )]
        struct UserProfile {
            #[companion(rename = "UserId")]
            id: u64,

            #[companion(rename = "DisplayName")]
            username: String,

            email: String,

            #[allow(dead_code)]
            #[companion(skip)]
            password_hash: String, // This field won't appear in companion enums

            age: Option<u8>,
            is_verified: bool,
        }

        let mut profile = UserProfile {
            id: 12345,
            username: "alice_dev".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: "secret_hash".to_string(),
            age: Some(28),
            is_verified: true,
        };

        // Use custom method names
        let user_id = profile.get_field(UserProfileField::UserId);
        assert_eq!(user_id, UserProfileValue::UserId(12345));

        // Update using custom method
        profile.set_field(UserProfileValue::DisplayName("alice_developer".to_string()));
        assert_eq!(profile.username, "alice_developer");

        // The password_hash field is skipped, so it doesn't appear in enums
        let fields = UserProfile::get_all_fields();
        assert_eq!(
            fields,
            &[
                UserProfileField::UserId,
                UserProfileField::DisplayName,
                UserProfileField::Email,
                UserProfileField::Age,
                UserProfileField::IsVerified
            ]
        );

        // Work with optional fields
        profile.set_field(UserProfileValue::Age(None));
        assert_eq!(profile.age, None);

        // Serialize/deserialize the values (if serde feature is enabled)
        let all_values = profile.as_values();
        for value in all_values {
            let serialized = serde_json::to_string(&value).unwrap();
            println!("Field value: {serialized}");
            if let UserProfileValue::UserId(_) = value {
                assert_eq!(serialized, r#"{"type":"userId","value":12345}"#);
            }
        }
    }

    #[test]
    fn test_try_from() {
        use std::convert::TryInto;

        #[derive(EnumCompanion)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        struct Test {
            name: String,
            distance: u32,
            speed: u32,
        }

        let test = Test {
            name: "Test".to_string(),
            distance: 42,
            speed: 100,
        };

        // Test successful conversion
        let name_value = test.value(TestField::Name);
        let name: String = name_value.clone().try_into().unwrap();
        assert_eq!(name, "Test".to_string());

        let distance_value = test.value(TestField::Distance);
        let distance: u32 = distance_value.clone().try_into().unwrap();
        assert_eq!(distance, 42);

        let speed_value = test.value(TestField::Speed);
        let speed: u32 = speed_value.clone().try_into().unwrap();
        assert_eq!(speed, 100);

        // Test failed conversion
        let name_value_fail = test.value(TestField::Name);
        let name_res: Result<u32, _> = name_value_fail.try_into();
        assert!(name_res.is_err());
    }

    #[test]
    fn test_try_from_tuple() {
        use std::convert::TryInto;

        #[derive(EnumCompanion)]
        #[allow(dead_code)]
        #[companion(derive_field(PartialEq, Debug), derive_value(Debug, PartialEq))]
        struct Test {
            name: String,
            distance: u32,
        }

        // Test successful conversion
        let name_tuple = (TestField::Name, "Test".to_string());
        let name_value: TestValue = name_tuple.try_into().unwrap();
        assert_eq!(name_value, TestValue::Name("Test".to_string()));

        let distance_tuple = (TestField::Distance, 42u32);
        let distance_value: TestValue = distance_tuple.try_into().unwrap();
        assert_eq!(distance_value, TestValue::Distance(42));

        // Test failed conversion
        let name_tuple_fail = (TestField::Name, 42u32);
        let name_res: Result<TestValue, _> = name_tuple_fail.try_into();
        assert!(name_res.is_err());
        assert_eq!(name_res.unwrap_err(), TestField::Name);
    }
}
