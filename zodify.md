```rust
zod! {
    const v = ""
}

#[zod(extend())]
pub struct X  {
    name: String,
    #[zod(expect)]
    age: Option<u32>,
}
```

```typescript
zod.object({
    name: zod.string(),
    age: zod.integer().number().positive().
})
```