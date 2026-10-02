# mxchro

Automatizador de macros de teclado para Windows que permite grabar, configurar y ejecutar secuencias de teclas con diferentes modos de activación.

## Características

- **Múltiples modos de activación**: Once (una vez), Hold (mantener), Toggle (alternar)
- **Hook de teclado global**: Captura teclas a nivel del sistema
- **Simulación de teclas**: Ejecuta secuencias de teclas con timing preciso
- **Control de macros**: Activar/desactivar macros en tiempo real
- **Bloqueo de teclas**: Opción de bloquear la tecla original
- **Filtrado de teclas inyectadas**: Evita loops infinitos

## Arquitectura del Backend

El backend está diseñado con una arquitectura en capas para maximizar la portabilidad y facilitar el mantenimiento.

```
src-tauri/src/
├── core/                          # Lógica de negocio pura
│   ├── types.rs                   # Tipos de dominio
│   └── macro_manager.rs           # Servicio principal
├── infrastructure/                # Implementaciones de plataforma
│   └── keyboard/
│       ├── hook.rs                # Hook de Windows (WH_KEYBOARD_LL)
│       ├── mapper.rs              # Mapeo de teclas VK ↔ String
│       └── simulator.rs           # Simulación de teclas (SendInput)
├── adapters/                      # Conectores framework ↔ core
│   └── commands.rs                # Adaptador Tauri → MacroManager
└── lib.rs                         # Entry point, wiring
```

### Capa Core (Lógica de Negocio)

La capa `core/` contiene la lógica de negocio pura, completamente independiente del framework y del sistema operativo.

#### `types.rs`

Define los tipos de dominio:

- **`Mode`**: Enum con los modos de activación (Toggle, Hold, Once)
- **`KeyEvent`**: Representa una acción (tecla con hold opcional, o sleep)
- **`MacroConfig`**: Configuración completa de una macro
- **`MacroSummary`**: Resumen de una macro para listar
- **`Action`**: Closure que ejecuta una acción

#### `macro_manager.rs`

Servicio principal que gestiona el ciclo de vida de las macros:

- **Gestión de macros**: Agregar, eliminar, activar, desactivar, listar
- **Ejecución de macros**: Maneja los modos de activación (Once, Hold, Toggle)
- **Control de threads**: Usa `Condvar` para threads eficientes que se duermen/despiertan
- **Eventos de teclado**: `on_key_down()` y `on_key_up()` procesan eventos del hook

**Estructuras principales**:

- **`MacroManager`**: Servicio público con toda la API
- **`MacroInstance`**: Instancia interna de una macro con su estado de ejecución
- **`RepeatState`**: Estado compartido para controlar threads de repetición (active, should_stop)

### Capa Infrastructure (Plataforma)

La capa `infrastructure/` contiene implementaciones específicas de Windows.

#### `keyboard/hook.rs`

Implementa el hook de teclado de bajo nivel de Windows:

- Instala un hook global `WH_KEYBOARD_LL`
- Captura todos los eventos de teclado del sistema
- Detecta teclas inyectadas (flag `LLKHF_INJECTED`)
- Notifica al `MacroManager` de cada evento
- Bloquea teclas si la macro lo requiere

#### `keyboard/mapper.rs`

Mapeo bidireccional entre códigos virtuales de Windows (VK) y strings:

- Singleton con `Lazy<KeyMapper>` para eficiencia
- Soporta letras A-Z, dígitos 0-9, F1-F24, teclas especiales, numpad, mouse, multimedia
- Usa `IndexMap` para mantener orden de inserción
- Métodos: `string_to_vk()`, `vk_to_string()`, `strings_vec()`

#### `keyboard/simulator.rs`

Simulación de teclas usando la API moderna `SendInput`:

- Envía teclas por scan code (más confiable que VK)
- Soporta duración de hold (tiempo entre keydown y keyup)
- Reemplaza la API deprecated `keybd_event`

### Capa Adapters (Conectores)

La capa `adapters/` conecta el framework (Tauri) con el core.

#### `commands.rs`

Adaptador que expone la API del `MacroManager` como comandos de Tauri:

- `get_available_keys()`: Lista todas las teclas disponibles
- `add_macro()`: Agrega una nueva macro
- `delete_macro()`: Elimina una macro
- `enable_macro()`: Activa una macro
- `disable_macro()`: Desactiva una macro
- `list_macros()`: Lista todas las macros

### Entry Point

#### `lib.rs`

- Inicializa el `MacroManager` como singleton global
- Instala el hook de teclado
- Registra los comandos de Tauri
- Arranca la aplicación

## Flujo de Ejecución

1. **Inicio**: Se crea el `MacroManager` y se instala el hook de teclado
2. **Usuario crea macro**: Frontend envía `MacroConfig` vía comando Tauri
3. **MacroManager procesa**: Convierte strings a VK codes, construye acciones, spawnea thread si está enabled
4. **Hook captura tecla**: Notifica al `MacroManager` con `on_key_down()`
5. **MacroManager ejecuta**: Según el modo (Once/Hold/Toggle), ejecuta la acción o activa el thread de repetición
6. **Thread de repetición**: Se despierta con `Condvar`, ejecuta acciones en loop, se duerme cuando se desactiva
7. **Simulación**: `Keyboard::press_key()` envía las teclas al sistema

## Portabilidad

La arquitectura en capas facilita la migración:

- **Cambiar framework** (Electron, GTK, etc.): Solo reescribir `adapters/` y `lib.rs`
- **Cambiar sistema operativo** (Linux, macOS): Solo reescribir `infrastructure/keyboard/`
- **Core**: Se mantiene intacto en todos los casos

## Tecnologías

- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS + shadcn/ui
- **Backend**: Rust + Windows API
- **Framework**: Tauri 2.0
- **Dependencias clave**:
  - `windows`: API de Windows para hooks y simulación
  - `indexmap`: Mapeo ordenado de teclas
  - `once_cell`: Singletons thread-safe
  - `serde`: Serialización para comunicación frontend-backend

## Desarrollo

```bash
# Instalar dependencias
pnpm install

# Desarrollo
pnpm tauri dev

# Build
pnpm tauri build
```

## Licencia

MIT
