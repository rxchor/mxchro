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
- [ ] Hotkey para activar/desactivar macros individuales (atajo de teclado global para controlar macros sin abrir la interfaz)
- [ ] Colecciones para agrupar macros (permite organizar macros y activar/desactivar grupos completos en masa)

## ✅ Completado

### Refactors de arquitectura
- **Persistencia JSON → PoloDB**: Migrado de archivo JSON a base de datos embebida PoloDB para mejor rendimiento y escalabilidad
- **Convenciones de nomenclatura**: Establecidas convenciones claras para Entity (dominio), DTO (transferencia), Document (BD), View (proyección)
- **Renombrado keyboard → input**: Refactorizado el módulo `keyboard` a `input` y `Keyboard` a `InputSimulator` para reflejar que soporta tanto teclado como ratón

### Features
- **Soporte para clicks de ratón**: Implementado soporte completo para botones de ratón (izquierdo, derecho, central, X1, X2) usando `INPUT_MOUSE` de Windows API

## 💡 Ideas / Backlog

- [ ] Hotkey global para pausar/resumir todas las macros temporalmente
- [ ] Notificaciones/toasts cuando macros se activan/desactivan
- [ ] Historial de ejecuciones de macros
- [ ] Estadísticas de uso de macros
