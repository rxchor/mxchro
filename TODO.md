# TODO - mxchro

## 🔴 Crítico / Bugs

- [ ] **WebView2 Hook**: Hook no captura eventos cuando la app tiene el foco (requiere listener frontend que envíe eventos al backend via Tauri command)

## 🟡 QoL / Mejoras de UI

- [x] **ActionsList scroll interno**: El listado de acciones crece hacia abajo cuando hay muchas. Debería tener scroll interno dentro del contenedor en lugar de expandirse infinitamente
- [x] **ActionsList elementos más compactos**: Los elementos del listado son muy altos. Cambiar el layout de columna a fila para hacerlos más finos y compactos. Considerar:
  - Icono + Tecla + Tipo en una sola línea horizontal
  - Reducir padding vertical
  - Mantener drag & drop funcional con el nuevo layout

## 🟢 Features futuras

- [ ] Grabación de macros en tiempo real (capturar pulsaciones y generar acciones automáticamente)
- [x] ~~Triggers compuestos~~ → Ya se pueden hacer combinando press/release de múltiples teclas (ej: Ctrl Press → Shift Press → A Press → A Release → Shift Release → Ctrl Release)
- [ ] Macros por aplicación (activar/desactivar según ventana en foco)
- [ ] Import/export de perfiles de macros
- [ ] Detección de teclas en UI (input que detecta tecla presionada en lugar de selección manual)
- [x] ~~Velocidad de repetición configurable para Hold/Toggle~~ → Se controla con WAITs en las acciones (ej: Click Press → Click Release → Wait 10ms = 100 clicks/segundo)
- [x] Hotkey para activar/desactivar macros individuales (atajo de teclado global para controlar macros sin abrir la interfaz)
- [x] ~~Colecciones para agrupar macros~~ → Implementado con soporte para crear/eliminar colecciones, mover macros entre colecciones, y activar/desactivar colecciones completas en masa
- [x] Hotkey para activar/desactivar colecciones completas (atajo de teclado global para controlar todas las macros de una colección)

## ✅ Completado

### Refactors de arquitectura
- **Persistencia JSON → PoloDB**: Migrado de archivo JSON a base de datos embebida PoloDB para mejor rendimiento y escalabilidad
- **Convenciones de nomenclatura**: Establecidas convenciones claras para Entity (dominio), DTO (transferencia), Document (BD), View (proyección)
- **Renombrado keyboard → input**: Refactorizado el módulo `keyboard` a `input` y `Keyboard` a `InputSimulator` para reflejar que soporta tanto teclado como ratón
- **Separación de responsabilidades**: Refactorizado `macro_manager.rs` en dos módulos separados:
  - `managers/macro_manager.rs`: Lógica de macros y procesamiento de eventos
  - `managers/collection_manager.rs`: Lógica de colecciones
  - Estado compartido mediante `Arc<RwLock<>>` entre ambos managers
  - `MacroManager` delega operaciones de colecciones a `CollectionManager`

### Features
- **Soporte para clicks de ratón**: Implementado soporte completo para botones de ratón (izquierdo, derecho, central, X1, X2) usando `INPUT_MOUSE` de Windows API
- **Toggle hotkeys**: Sistema completo de hotkeys globales para activar/desactivar macros individuales y colecciones completas desde cualquier aplicación
  - Componente `KeyInput` reutilizable para capturar teclas
  - UI integrada en MacroForm y CollectionSidebar
  - Badges visuales mostrando hotkeys configurados
  - Detección automática en `process_key_event` del hook de teclado
- **Sistema de versionado de esquema**: Detección automática de cambios en la estructura de datos con reset automático de la BD y variable de entorno `RESET_DB=true` para desarrollo

## 💡 Ideas / Backlog

- [ ] Hotkey global para pausar/resumir todas las macros temporalmente
- [ ] Notificaciones/toasts cuando macros se activan/desactivan
- [ ] Historial de ejecuciones de macros
- [ ] Estadísticas de uso de macros
