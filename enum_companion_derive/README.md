# Enum Companion Derive

This crate provides a procedural macro to derive two companion enums for a given struct, along with several helper methods. This is useful for dynamically accessing and updating struct fields.

## Basic Usage

Here's a simple example of how to use `EnumCompanion`:

```rust,ignore
# use enum_companion_derive::EnumCompanion;
# use enum_companion_trait::EnumCompanionTrait;
#[derive(EnumCompanion)]
#[companion(derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
struct MyStruct {
    id: u32,
    name: String,
}

// The macro generates two enums: `MyStructField` and `MyStructValue`.
// It also generates `value()` and `update()` methods.

let mut my_struct = MyStruct {
    id: 1,
    name: "Example".to_string(),
};

// Access a field's value
let name_value = my_struct.value(MyStructField::Name);
assert_eq!(name_value, MyStructValue::Name("Example".to_string()));

// Update a field's value
my_struct.update(MyStructValue::Id(10));
assert_eq!(my_struct.id, 10);

// Get all fields
let fields = MyStruct::fields();
assert_eq!(fields, &[MyStructField::Id, MyStructField::Name]);
```

The macro generates the following code behind the scenes (without any `#[companion]` attributes):

```rust,ignore
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MyStructField {
    Id,
    Name,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MyStructValue {
    Id(u32),
    Name(String),
}

impl MyStruct {
    pub fn value(&self, field: MyStructField) -> MyStructValue {
        // ...
    }

    pub fn update(&mut self, value: MyStructValue) {
        // ...
    }

    pub fn fields() -> [MyStructField; 2] {
        // ...
    }

    pub fn as_values(&self) -> Vec<MyStructValue> {
        // ...
    }
}

impl std::str::FromStr for MyStructField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ...
    }
}
```

## Full-Featured Example

`EnumCompanion` offers several attributes to customize its behavior.

```rust
# use enum_companion_derive::EnumCompanion;
# use uuid::Uuid;
#[derive(EnumCompanion)]
#[companion(
    value_fn = "get_value",
    update_fn = "set_value",
    fields_fn = "get_fields",
    derive_field(PartialEq, Eq, Hash, Debug),
    derive_value(Debug, PartialEq, Eq, Hash)
)]
struct Config {
    app_name: String,
    #[companion(rename = "Version")]
    app_version: (u8, u8, u8),
    #[companion(skip)]
    session_id: Uuid,
}

let mut config = Config {
    app_name: "My Awesome App".to_string(),
    app_version: (1, 0, 0),
    session_id: Uuid::new_v4(),
};

// Use the custom function names
let version = config.get_value(ConfigField::Version);
assert_eq!(version, ConfigValue::Version((1, 0, 0)));

// The `session_id` field was skipped
assert_eq!(
    Config::get_fields(),
    [ConfigField::AppName, ConfigField::Version]
);

// The derived traits can be used
use std::collections::HashSet;
let mut field_set = HashSet::new();
field_set.insert(ConfigField::AppName);
```

### Converting Values Back

The generated `...Value` enum also implements `TryFrom<...Value>` for each of the underlying types, allowing you to convert a value enum back into its concrete type.

```rust
# use enum_companion_derive::EnumCompanion;
# use std::convert::TryInto;
# #[derive(EnumCompanion)]
# // fields_fn="get_fields" is only needed here to avoid using the trait from the enum_companion crate (not available here)
# #[companion(fields_fn = "get_fields",derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
# struct MyStruct {
#     id: u32,
#     name: String,
# }
# let my_struct = MyStruct { id: 1, name: "Example".to_string() };
let name_value = my_struct.value(MyStructField::Name);

// Convert the value back into a String
let name: String = name_value.try_into().unwrap();
assert_eq!(name, "Example".to_string());

// This would fail to compile if you tried to convert to the wrong type,
// but a runtime check is also possible.
let id_value = my_struct.value(MyStructField::Id);
let id_res: Result<String, _> = id_value.try_into();
assert!(id_res.is_err());
```

### Creating Values from Tuples

You can also create a `...Value` enum from a tuple of `(...Field, InnerValue)`, which can be useful for constructing values dynamically.

```rust
# use enum_companion_derive::EnumCompanion;
# use std::convert::TryInto;
# #[derive(EnumCompanion)]
# // fields_fn="get_fields" is only needed here to avoid using the trait from the enum_companion crate (not available here)
# #[companion(fields_fn = "get_fields",derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
# struct MyStruct {
#     id: u32,
#     name: String,
# }
let name_tuple = (MyStructField::Name, "Example".to_string());
let name_value: MyStructValue = name_tuple.try_into().unwrap();
assert_eq!(name_value, MyStructValue::Name("Example".to_string()));

// This would fail if the inner value type does not match the field.
let id_tuple_fail = (MyStructField::Name, 42u32);
let id_res: Result<MyStructValue, _> = id_tuple_fail.try_into();
assert!(id_res.is_err());
```

### Available Attributes

**On the struct:**

- `#[companion(value_fn = "new_name")]`: Changes the name of the value getter function (default: `"value"`).
- `#[companion(update_fn = "new_name")]`: Changes the name of the value setter function (default: `"update"`).
- `#[companion(fields_fn = "new_name")]`: Changes the name of the fields getter function (default: `"fields"`).
- `#[companion(derive_field(Trait1, Trait2))]`: Adds derive macros to the `...Field` enum.
- `#[companion(derive_value(Trait1, Trait2))]`: Adds derive macros to the `...Value` enum.
- `#[companion(serde_field(attr=value))]`: Adds Serde attributes to the `...Field` enum.
- `#[companion(serde_value(attr=value))]`: Adds Serde attributes to the `...Value` enum.

**On fields:**

- `#[companion(rename = "NewName")]`: Renames the variant in the companion enums.
- `#[companion(skip)]`: Excludes the field from the companion enums.

## Trait-based Access

If the default method names (`value`, `update`, `fields`) are not overridden, the macro will also implement the `enum_companion::EnumCompanionTrait`. This allows for generic programming over any struct that derives `EnumCompanion` with default settings.

## Limits

This crate has those known limitations :

- **`Clone` Requirement**: The `value()` method needs to clone the field values. Therefore, all fields in the struct must implement the `Clone` trait.
- **Named Structs Only**: The macro can only be used on structs with named fields (e.g., `struct MyStruct { id: u32 }`). It does not support tuple structs or unit structs.
- **`TryFrom` with Generics**: Due to Rust's orphan rule, `TryFrom` is not implemented for fields that are generic or contain generic types.
