# rust-angular-units

Feature-rich library for representing and manipulating angular quantities. Provides type-safe wrapper types
for each unit as well as helper traits for abstracting over concrete types. Conversions between types is
easy and safe, allowing highly flexible manipulation.

## Provided Units:
* Degrees - `Deg<T>`
* Radians - `Rad<T>`
* Turns - `Turns<T>` (1 turn is a full rotation)
* Arc minutes - `ArcMinutes<T>`
* Arc seconds - `ArcSeconds<T>`

## Usage:

Add this to your `Cargo.toml`:

```toml
[dependencies]
num = "0.1"
```
## Examples

* Converting from Degrees to Radians:
```rust
  let angle = Deg(45.0);
  let radians: Rad<_> = angle.into_angle();
```

* Composing angles from multiple units:
```rust
  let degrees: Deg<f32> = Deg(50.0_f32) + ArcMinutes(25.0_f32) + Rad(std::f32::consts::PI / 6.0_f32);
```
