# Enum Companion

[![Crates.io Version](https://img.shields.io/crates/v/enum_companion?label=crates.io)](https://crates.io/crates/enum_companion) [![docs.rs](https://img.shields.io/docsrs/enum_companion)](https://docs.rs/enum_companion/latest/enum_companion/) [![GitHub License](https://img.shields.io/github/license/solidev/enum_companion)](./LICENSE) [![GitHub contributors](https://img.shields.io/github/contributors/solidev/enum_companion)](#)

A Rust procedural macro that generates companion enums for structs, enabling dynamic field access and updates. This crate provides type-safe runtime reflection capabilities for struct fields.

> ⚠️ **Work In Progress:** This crate is currently a big WIP (Work In Progress). The API is unstable and may change significantly. Use at your own risk!

## Adding to Your Project

Add this to your `Cargo.toml`:

```toml
[dependencies]
enum_companion = "0.1.2"
```

Or using cargo:

```bash
cargo add enum_companion
```

## Macro Summary

The `#[derive(EnumCompanion)]` macro generates:

- **Field Enum** (`{StructName}Field`): An enum representing all struct fields
- **Value Enum** (`{StructName}Value`): An enum containing the typed values of each field
- **Helper Methods**:
  - `value(field: {StructName}Field) -> {StructName}Value`: Get a field's value
  - `update(&mut self, value: {StructName}Value)`: Update a field's value
  - `fields() -> &'static [{StructName}Field]`: Get all field enum variants as an array
  - `as_values(&self) -> Vec<{StructName}Value>`: Get all field values as a vector
- **`FromStr` Implementation**: The `{StructName}Field` enum implements `FromStr` to allow conversion from a string.
- **`EnumCompanionTrait`**: A trait providing a generic interface to the companion methods, implemented automatically **when default method names are used**.
- **`EnumCompanionField`** trait: These traits are implemented for the generated enums.

## Traits

### `EnumCompanionTrait<F, V>`

If the default method names (`value`, `update`, `fields`) are not overridden, the macro will also implement the `EnumCompanionTrait`. This allows for generic programming over any struct that derives `EnumCompanion` with default settings.

### `EnumCompanionField`

This trait is implemented for the generated field enum. It provides methods to get the field name, value type, and other metadata :

- `name(&self) -> &str`: Get the field name.
- `type_str(&self) -> &str`: Get the field value type name.
- `title(&self) -> &str`: Get the field title.
- `description(&self) -> &str`: Get the field description.
- `order(&self) -> u32`: Get the field order.

### Attributes

**On the struct:**

- `#[companion(value_fn = "custom_name")]`: Customize the value getter method name.
- `#[companion(update_fn = "custom_name")]`: Customize the value setter method name.
- `#[companion(fields_fn = "custom_name")]`: Customize the fields getter method name.
- `#[companion(derive_field(Trait1, Trait2))]`: Add derives to the field enum.
- `#[companion(derive_value(Trait1, Trait2))]`: Add derives to the value enum.
- `#[companion(serde_field(Attribute))]`: Add Serde attributes to the field enum.
- `#[companion(serde_value(Attribute))]`: Add Serde attributes to the value enum.

**On fields:**

- `#[companion(skip)]`: Skip a field from companion enum generation.
- `#[companion(rename = "NewName")]`: Rename the enum variant for a field.
- `#[companion(title="Title")]`: Set a title for the field (defaults to the field name).
- `#[companion(description="Description")]`: Set a description for the field (defaults to an empty string).
- `#[companion(order=1)]`: Set the order for the field (defaults to 0).

## Examples

### Basic Example

```rust
# use enum_companion::EnumCompanion;
# use crate::enum_companion::EnumCompanionField;
#[derive(EnumCompanion)]
#[companion(derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
struct Person {
    id: u32,
    name: String,
    age: u8,
}

fn main() {
    let mut person = Person {
        id: 1,
        name: "Alice".to_string(),
        age: 30,
    };

    // Access field values dynamically
    let name_value = person.value(PersonField::Name);
    assert_eq!(name_value, PersonValue::Name("Alice".to_string()));

    // Update field values dynamically
    person.update(PersonValue::Age(31));
    assert_eq!(person.age, 31);

    // Get all fields
    let fields = Person::fields();
    assert_eq!(fields, &[PersonField::Id, PersonField::Name, PersonField::Age]);

    // Get all values as a vector
    let all_values = person.as_values();
    println!("All values: {:?}", all_values);

    // Get a field title, description and type as strings
    let field_title = PersonField::Name.title();
    let field_description = PersonField::Name.description();
    let field_type = PersonField::Name.type_str();
    println!("Field Title: {}, Description: {}, Type: {}", field_title, field_description, field_type);
}
```

### Converting Values Back

The generated `Value` enum implements `TryFrom<Value>` for each of the underlying types. This allows you to easily convert a `Value` enum back into a concrete type.

```rust
# use enum_companion::{EnumCompanion, EnumCompanionTrait};
# use std::convert::TryInto;

#[derive(EnumCompanion)]
#[companion(derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
struct ServerConfig {
    host: String,
    port: u16,
}

fn main() {
    let config = ServerConfig {
        host: "localhost".to_string(),
        port: 8080,
    };

    // Get a value from the struct
    let port_value = config.value(ServerConfigField::Port);

    // Convert the value back into a u16
    let port_u16: u16 = port_value.try_into().expect("Should be a u16");
    assert_eq!(port_u16, 8080);

    // Trying to convert to the wrong type will fail
    let host_value = config.value(ServerConfigField::Host);
    let host_res: Result<u16, _> = host_value.try_into();
    assert!(host_res.is_err());
}
```

### Creating Values from Tuples

You can also create a `Value` enum from a tuple of `(Field, InnerValue)`, which can be useful for constructing values dynamically.

```rust
# use enum_companion::{EnumCompanion, EnumCompanionTrait};
# use std::convert::TryInto;

#[derive(EnumCompanion)]
#[companion(derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
struct MyStruct {
    id: u32,
    name: String,
}

let name_tuple = (MyStructField::Name, "Example".to_string());
let name_value: MyStructValue = name_tuple.try_into().unwrap();
assert_eq!(name_value, MyStructValue::Name("Example".to_string()));

// This would fail if the inner value type does not match the field.
let id_tuple_fail = (MyStructField::Name, 42u32);
let id_res: Result<MyStructValue, _> = id_tuple_fail.try_into();
assert!(id_res.is_err());

```

> **Limitation**: Due to Rust's orphan rule, `TryFrom` is not implemented for fields that are generic or contain generic types.

### The `EnumCompanionTrait`

When you use the default method names (`value`, `update`, `fields`), the macro will also implement the `enum_companion::EnumCompanionTrait` for your struct. This trait provides a generic way to interact with any struct that uses `EnumCompanion`.

```rust
use enum_companion::{EnumCompanion, EnumCompanionTrait};

#[derive(EnumCompanion)]
#[companion(derive_field(Debug), derive_value(Debug))]
struct MyStruct {
    foo: i32,
    bar: String,
}

fn process_any_companion<T, F, V>(companion: &T)
where
    T: EnumCompanionTrait<F, V>,
    F: Copy + std::fmt::Debug + 'static,
    V: std::fmt::Debug,
{
    println!("Processing fields...");
    for &field in T::fields() {
        let value = companion.value(field);
        println!("  {:?}: {:?}", field, value);
    }
}

let my_struct = MyStruct { foo: 42, bar: "hello".to_string() };
process_any_companion(&my_struct);

```

### The `EnumCompanionField` Trait

The generated field enum implements the `EnumCompanionField` trait, which provides methods to get the field name, value type, and other metadata:

```rust
use enum_companion::{EnumCompanion, EnumCompanionField};
#[derive(EnumCompanion)]
#[companion(derive_field(Debug, PartialEq), derive_value(Debug, PartialEq))]
struct Example {
    #[companion(title="Identifier", description="Id Description", order=1)]
    id: u32,
    name: String,
}
let field = ExampleField::Id;
assert_eq!("id", field.name());
assert_eq!("u32", field.type_str());
assert_eq!("Identifier", field.title());
assert_eq!("Id Description", field.description());
assert_eq!(1, field.order());

```

### Full Example with Attributes

```rust
use enum_companion::{EnumCompanion, EnumCompanionTrait, EnumCompanionField};
use serde::{Serialize, Deserialize};

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

    #[companion(title="Mail", description="User's email address", order=2)]
    email: String,

    #[companion(skip)]
    password_hash: String,  // This field won't appear in companion enums

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
    println!("Field value: {}", serialized);
    if let UserProfileValue::UserId(_) = value {
        assert_eq!(serialized, r#"{"type":"userId","value":12345}"#);
    }
}

// Get field infos
let field = UserProfileField::DisplayName;
assert_eq!(field.name(), "DisplayName");
assert_eq!(field.type_str(), "String");
assert_eq!(field.title(), "DisplayName");
assert_eq!(field.description(), "");
assert_eq!(field.order(), 0);

let field = UserProfileField::Email;
assert_eq!(field.name(), "email");
assert_eq!(field.type_str(), "String");
assert_eq!(field.title(), "Mail");
assert_eq!(field.description(), "User's email address");
assert_eq!(field.order(), 2);


```

## Generated Code

For a basic struct like:

```rust,ignore
#[derive(EnumCompanion)]
#[companion(derive_value(Debug, PartialEq))]
struct Example {
    id: u32,
    name: String,
}
```

The macro generates:

```rust,ignore
// Recursive expansion of EnumCompanion macro
// ===========================================

#[doc = r" An enum representing the fields of the struct."]
#[allow(dead_code)]
#[derive(Copy, Clone)]
enum ExampleField {
    Id,
    Name,
}
impl ExampleField {
    pub const FIELDS: &'static [ExampleField] = &[ExampleField::Id, ExampleField::Name];
}
impl ::enum_companion::EnumCompanionField for ExampleField {
    fn name(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
    fn type_str(&self) -> &'static str {
        match self {
            Self::Id => "u32",
            Self::Name => "String",
        }
    }
    fn title(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::Id => "",
            Self::Name => "",
        }
    }
    fn order(&self) -> u32 {
        match self {
            Self::Id => 0u32,
            Self::Name => 0u32,
        }
    }
}
#[doc = r" An enum representing the values of the struct's fields."]
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
enum ExampleValue {
    Id(u32),
    Name(String),
}
impl ::enum_companion::EnumCompanionValue for ExampleValue {
    fn field_name(&self) -> &'static str {
        match self {
            Self::Id(_) => "id",
            Self::Name(_) => "name",
        }
    }
    fn type_name(&self) -> &'static str {
        match self {
            Self::Id(_) => "u32",
            Self::Name(_) => "String",
        }
    }
}
impl std::str::FromStr for ExampleField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "id" | "Id" => Ok(Self::Id),
            "name" | "Name" => Ok(Self::Name),
            _ => Err(format!("Invalid field name: {}", s)),
        }
    }
}
impl Example {
    #[doc = r" Returns an array of all field enum variants."]
    pub fn fields() -> &'static [ExampleField] {
        ExampleField::FIELDS
    }
    #[doc = r" Returns a vector of all field values."]
    pub fn as_values(&self) -> Vec<ExampleValue> {
        Self::fields()
            .iter()
            .map(|&field| self.value(field))
            .collect()
    }
    #[doc = r" Returns the value of a specific field."]
    pub fn value(&self, field: ExampleField) -> ExampleValue {
        match field {
            ExampleField::Id => ExampleValue::Id(self.id.clone()),
            ExampleField::Name => ExampleValue::Name(self.name.clone()),
        }
    }
    #[doc = r" Updates the value of a specific field."]
    pub fn update(&mut self, value: ExampleValue) {
        match value {
            ExampleValue::Id(value) => self.id = value,
            ExampleValue::Name(value) => self.name = value,
        }
    }
}
impl ::enum_companion::EnumCompanionTrait<ExampleField, ExampleValue> for Example {
    fn value(&self, field: ExampleField) -> ExampleValue {
        self.value(field)
    }
    fn update(&mut self, value: ExampleValue) {
        self.update(value)
    }
    fn fields() -> &'static [ExampleField] {
        &ExampleField::FIELDS
    }
    fn as_values(&self) -> Vec<ExampleValue> {
        self.as_values()
    }
}
impl std::convert::TryFrom<ExampleValue> for u32 {
    type Error = ExampleValue;
    fn try_from(value: ExampleValue) -> Result<Self, Self::Error> {
        match value {
            ExampleValue::Id(val) => Ok(val),
            _ => Err(value),
        }
    }
}
impl std::convert::TryFrom<ExampleValue> for String {
    type Error = ExampleValue;
    fn try_from(value: ExampleValue) -> Result<Self, Self::Error> {
        match value {
            ExampleValue::Name(val) => Ok(val),
            _ => Err(value),
        }
    }
}
impl std::convert::TryFrom<(ExampleField, u32)> for ExampleValue {
    type Error = ExampleField;
    fn try_from(value: (ExampleField, u32)) -> Result<Self, Self::Error> {
        let (field, value) = value;
        match field {
            ExampleField::Id => Ok(ExampleValue::Id(value)),
            _ => Err(field),
        }
    }
}
impl std::convert::TryFrom<(ExampleField, String)> for ExampleValue {
    type Error = ExampleField;
    fn try_from(value: (ExampleField, String)) -> Result<Self, Self::Error> {
        let (field, value) = value;
        match field {
            ExampleField::Name => Ok(ExampleValue::Name(value)),
            _ => Err(field),
        }
    }
}
```

## Use Cases

- **Dynamic forms**: Build forms that can handle any struct type
- **Serialization helpers**: Generic serialization without knowing field types at compile time
- **Configuration management**: Update struct fields from external configuration
- **API endpoints**: Generic CRUD operations over struct fields
- **Testing utilities**: Compare and manipulate struct fields generically

## Limitations

- **`Clone` Requirement**: The `value()` method needs to clone the field values. Therefore, all fields in the struct must implement the `Clone` trait.
- **Named Structs Only**: The macro can only be used on structs with named fields (e.g., `struct MyStruct { id: u32 }`). It does not support tuple structs or unit structs.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
