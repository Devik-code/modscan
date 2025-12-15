# Preparacion

instalar rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

comprobar que las variables de entorno quedan ok, revisar doc oficial en caso de
cualquier problema.

# Requerimientos

- rust
- cross (para compilacion cruzada)
- docker (requerimiento de cross)

# Compilar

## notebooks architecture x86_64 (ubuntu)

```
cargo build --release
```

(como se instala en ubuntu, esto funciona si se compila en ubuntu, en archlinux por
ej que tiene versiones mas nuevas de libc, etc, no funcionara, ahi ocupar cross)

```
cross build --release --target x86_64-unknown-linux-gnu
```

## rpi zero

```
cross build --release --target arm-unknown-linux-gnueabihf
```

## rpi 4

```
cross build --release --target armv7-unknown-linux-gnueabihf
```

# Generar .deb

Ocuparemos cargo deb para generar los .deb. TENER YA COMPILADOS LOS BINARIOS.

- ubuntu: `cargo deb --no-build --target x86_64-unknown-linux-gnu -v`
- rpi zero: `cargo deb --no-build --target arm-unknown-linux-gnueabihf -v --no-strip`
- rpi 4: `cargo deb --no-build --target armv7-unknown-linux-gnueabihf -v --no-strip`

el deb quedara en `target/<arch>/debian/CARGO_PKG_NAME_<version>_<arch>.deb`
# modscan