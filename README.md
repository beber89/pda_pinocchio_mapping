# Mapping Library for Pinocchio Programs

A lightweight, `no_std` utility crate for creating a solidity-like mapping for pinocchio programs.

This library provides a simple abstraction called **`Mapping`**, which allows programs to create and overwrite PDA-derived accounts.


---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pda-pinocchio-mapping = "0.x"
```

This crate depends only on:

- `pinocchio`
- `pinocchio_system`
- `pinocchio_pubkey`
- `bytemuck`

and remains fully `no_std`.

---

## 🔧 Required Trait


### `Bumpy`

Supplies the bump used in PDA derivation. Your struct type should be implementing this trait (i.e. contains `bump` value).

```rust
pub trait Bumpy {
    fn bump(&self) -> u8 {self.bump};
}
```

---

## 🧱 The `Mapping` Abstraction

A `Mapping` is created by convenience macro:

```rust
let mapping = mapping!(b"positions", payer);
```

It stores and retrieves values associated with:

- a **key** (`Pubkey`)
- a **value** (`T : Pod + Bumpy`)

PDA seeds are constructed using:

```
[name, key, [bump]]
```

This is abstracted behind `Mapping`.

The `mapping!` macro:

- Validates at compile time that `crate::ID` exists  
- Invokes `Mapping::new(&crate::ID, name, payer)`  

---

## 📌 Core Methods

### `create`

Creates a PDA for the first time and initializes it to `value`.

```rust
mapping.create(&user_key, value, account_info)?;
```

Fails if the account already exists.

---

### `set`

Creates *or* overwrites a PDA.

```rust
mapping.set(&user_key, value, account_info)?;
```

- If the PDA exists → overwrite  
- If not → create + initialize  

---

### `update`

Overwrites only if the PDA exists.

```rust
mapping.update(&user_key, new_value, account_info)?;
```

Fails if the PDA is uninitialized.

---

## 📘 Example:

```rust
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Position {
    pub amount: u64,
    pub bump: u8,
}

impl Bumpy for Position {
    fn bump(&self) -> u8 { self.bump }
}
```

### Creating or updating a position

```rust

let [position_account, accounts @..] = accounts;
let position = Position { amount: 100, bump };

let mapping = mapping!(b"positions", payer);
mapping.set(&user.pubkey(), position, position_account)?;
```

---


## 🔒Validations and Checks 

This library ensures:

- PDA account must match the derived address
- Account data size must equal `core::mem::size_of<T>()`
- Only the rightful program owner may update stored values
- PDA creation requires amount of **rent-exempt lamports**


