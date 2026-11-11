# Tests del backend

Los tests unitarios están agrupados en `unit/`:

- `managers.rs`: persistencia, concurrencia, índices de entrada y toggles.
- `key_mapper.rs`: conversión entre nombres de teclas y códigos virtuales.

Los módulos de producción los incluyen con `#[cfg(test)]` y `#[path]` para
conservar el acceso a sus detalles internos sin ampliar la API pública.
No se compilan en las builds normales.

Ejecutar desde `src-tauri`:

```sh
cargo test --locked
```

Los tests de managers usan bases de datos temporales aisladas y no generan
pulsaciones reales.
