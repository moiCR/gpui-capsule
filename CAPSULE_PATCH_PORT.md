# Portado de capsule-patch a gpui-ce

Origen: `gpui-capsule/capsule-patch`, hasta `ec16790e13`.
Base de destino: `81c2e755f75badfd2722765bc1a89583ea9423fa`.
Rama local: `codex/capsule-patch`. Sin push ni cambios de dependencias o lockfile.

## Correspondencia con el origen

| Commits originales | Comportamiento adaptado |
| --- | --- |
| `649732a5c8`, `bd65f64950` | Interactividad de teclado dinámica de layer-shell; solicitudes y commits únicamente al cambiar el modo. |
| `51a8431e8b` | Arrastre externo inmediato y formatos URI, archivos GNOME y texto en Wayland. |
| `5c56656c6e` | Miniatura de 48 × 48 o icono de archivo/directorio para el arrastre nativo. |
| `f8866e713f` | Anunciar únicamente la acción Copy. |
| `76890cf171` | Respetar ambos ejes explícitos de imágenes y probar el recorte vertical. |
| `55db950882`, `f13e0072ee`, `ec16790e13` | ext-session-lock-v1, configure antes del primer commit y desbloqueo antes de destruir la última superficie. |

Se excluyen `604862b139` (poda del repositorio) y `7984023241` (actualización global del lockfile).

## Compatibilidad y adaptación

- `AnyDrag` conserva sus campos públicos. El modo inmediato se guarda internamente en la interactividad y el gesto, usando la implementación existente de `on_drag_alt`.
- Se añaden `immediate_external_drag_payload`, `Window::set_keyboard_interactivity`, el módulo `session_lock` y `WindowKind::SessionLock`. Los matches exhaustivos externos de `WindowKind` pueden necesitar la nueva variante cuando compilen para Linux con Wayland.
- La resolución del payload ocurre como máximo una vez. Se conservan rechazo del backend, arrastre interno y restauración del arrastre en la ventana de origen; el modo inmediato se limpia al terminar, cancelar, restaurar o comenzar otro gesto.
- Los buffers del icono usan ARGB premultiplicado en el orden nativo de bytes. Se reutiliza memfd, se comprueban dimensiones y se asigna el rol de icono antes de su primer commit. La limpieza de superficie, buffer y data source se realiza mediante Drop, incluyendo cancelación y cierre del cliente.
- El lock comparte una solicitud entre salidas y rechaza superficies duplicadas para una salida. Cerrar una ventana secundaria mantiene el lock; cerrar la última desbloquea una sola vez antes de destruirla.
- Una solicitud pendiente espera Locked o Finished antes de elegir el destructor válido. Se ignoran eventos de locks anteriores. Los errores de creación limpian los objetos de superficie y viewport.
- Las superficies del lock no emiten commits antes del configure reconocido y no aplican las operaciones de ocultación de ventanas xdg. X11 devuelve un error para SessionLock.

## Validación

Resultados obtenidos antes del reinicio del equipo:

| Verificación | Base | Portado |
| --- | --- | --- |
| Pruebas de gpui-ce | 402 pasan | 404 pasan |
| Pruebas del backend Linux | 54 pasan | 61 pasan |
| `just clippy` del workspace, todos los targets, con warnings como errores | Pasa | Pasa |
| Núcleo sin características predeterminadas | Pasa | Pasa |
| Backend X11 sin Wayland | Pasa | Pasa |
| Backend Wayland sin X11 | Pasa | Pasa |
| Capsule en copia con dependencias locales de gpui-ce | 18 errores existentes en ui | Los mismos 18 errores en ui |

Se ejecutaron las mismas órdenes de pruebas, Clippy y matriz en la base y el portado. Las nueve pruebas nuevas cubren imágenes, promoción inmediata y resolución única del arrastre, rechazo y restauración, formatos de rutas, premultiplicación, iconos de respaldo y estados pendientes/adquiridos/rechazados del lock.

Órdenes de comparación (base extraída de Git y rama portadora):

```sh
cargo test -p gpui-ce -p gpui_ce_linux --lib --features gpui/test-support
just clippy
cargo check --locked -p gpui-ce --no-default-features --lib
cargo check --locked -p gpui_ce_linux --no-default-features --features x11 --lib
cargo check --locked -p gpui_ce_linux --no-default-features --features wayland --lib
```

En la rama portadora también se comprueba `cargo fmt --all -- --check` y `cargo check --locked --workspace --all-targets -j1`. Las herramientas auxiliares y el enlace de desarrollo de libxkbcommon-x11 se extrajeron localmente; no se instalaron paquetes del sistema.

La prueba ignorada `wayland_capsule_patch_lifecycle` se ejecutó en Sway headless con dos salidas y un XDG_RUNTIME_DIR independiente de la sesión del usuario. Pasaron los escenarios `lock`, `pending` y `layer`.

La traza del escenario lock confirmó tres superficies con ack_configure antes de cualquier commit, dos locks con un solo unlock_and_destroy cada uno, conservación del lock al cerrar la ventana secundaria y desbloqueo antes de destruir la última superficie. La traza de layer-shell mostró solo tres solicitudes de interactividad: la inicial y dos cambios, pese a repetir cada setter tres veces.

Tras el reinicio se repitieron las 465 pruebas unitarias desde sus binarios, `cargo check --locked --workspace --all-targets -j1`, `just clippy` y formato: todos pasan. Los registros nuevos se conservan en `target/capsule-port-validation/`; los registros anteriores de /tmp desaparecieron al reiniciar.

`cargo build --locked --workspace --all-targets -j1` también pasa, incluyendo ejemplos y binarios de pruebas, en 10 min 49 s. Se ejecutó dentro de un scope de systemd con MemoryMax=4G, CPUQuota=100%, TasksMax=128, prioridad nice 15 y afinidad a dos CPUs. No se obtuvo un resultado comparable de compilación completa en la base; la comparación anterior sí incluye chequeos de características, Clippy y pruebas.

## Límites de la verificación

- Capsule sigue presentando incompatibilidades previas de migración: opacity y construcción/conversión de colores, y DefiniteLength::Fraction. Su crate ui impide comprobar por compilación los consumidores posteriores. Este portado no modifica el código ni las dependencias del checkout de Capsule.
- macOS, Windows, WASM y MSRV 1.95 no se verificaron como targets nativos: solo está instalado x86_64-unknown-linux-gnu con Rust 1.98.1. Un chequeo del workspace en Linux no sustituye esas comprobaciones.
- Quedan pendientes las pruebas reales de rechazo por otro cliente, fallo de creación del renderer y transferencia/cancelación del icono de arrastre entre aplicaciones. Sus rutas se revisaron y los estados del lock tienen pruebas unitarias.
- Después del problema de estabilidad del equipo no se repiten pruebas gráficas. La compilación final usa un proceso, prioridad baja, cuota de un núcleo y un límite de 4 GB de memoria.
