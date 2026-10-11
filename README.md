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

Define los tipos de dominio siguiendo las convenciones de arquitectura limpia:

**Entities (Dominio puro):**
- **`Mode`**: Enum con los modos de activación (Toggle, Hold, Once)
- **`Action`**: Enum con las acciones posibles (Key o Wait)
- **`Macro`**: Configuración completa de una macro
- **`ActionCallback`**: Alias de tipo para closures que ejecutan acciones

**Views (Proyecciones para UI):**
- **`MacroView`**: Proyección parcial de una macro para listar (sin las acciones completas)

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
2. **Usuario crea macro**: Frontend envía `Macro` vía comando Tauri
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

## Arquitectura del Frontend

El frontend está organizado siguiendo las mismas convenciones de nomenclatura:

```
src/
├── model/                         # Tipos de dominio (camelCase)
│   ├── Macro.ts                   # Entity: Macro
│   ├── Action.ts                  # Entity: Action
│   ├── Mode.ts                    # Entity: Mode
│   └── MacroView.ts               # View: MacroView
├── services/                      # Servicios y API
│   ├── MacroService.ts            # Conversión Response → Model
│   └── KeyListService.ts          # Mapeo de teclas
├── components/                    # Componentes UI
│   ├── ui/                        # Componentes shadcn/ui
│   └── ...
└── pages/                         # Páginas de la aplicación
    ├── home/
    └── create_macro/
```

**Convención de conversión:**
```
Backend: Macro (snake_case en JSON)
    ↓
Frontend: MacroResponse (snake_case interface)
    ↓ fromSnakeCase()
Frontend: Macro (camelCase model para UI)
```

## Tecnologías

- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS + shadcn/ui
- **Backend**: Rust + Windows API
- **Framework**: Tauri 2.0
- **Dependencias clave**:
  - `windows`: API de Windows para hooks y simulación
  - `polodb_core`: Base de datos embebida (documentos BSON sobre RocksDB)
  - `indexmap`: Mapeo ordenado de teclas
  - `once_cell`: Singletons thread-safe
  - `serde`: Serialización para comunicación frontend-backend

## Requisitos

Para desarrollar y compilar la aplicación necesitas:

### Windows

1. **Rust**: Instala desde [rustup.rs](https://rustup.rs/)
2. **Node.js**: Versión 18 o superior desde [nodejs.org](https://nodejs.org/)
3. **pnpm**: Gestor de paquetes
   ```bash
   npm install -g pnpm
   ```
4. **Microsoft Visual C++ Build Tools**: Descarga desde [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Durante la instalación, selecciona "Desktop development with C++"
5. **LLVM**: Necesario para compilar RocksDB (backend de PoloDB)
   - Descarga desde [LLVM releases](https://github.com/llvm/llvm-project/releases)
   - Archivo: `LLVM-XX.X.X-win64.exe`
   - **Importante**: Marca "Add LLVM to the system PATH for all users" durante la instalación
   - Reinicia tu terminal después de instalar

### Verificar instalación

```bash
rustc --version      # Rust
node --version       # Node.js
pnpm --version       # pnpm
```

## Desarrollo

```bash
# Instalar dependencias
pnpm install

# Desarrollo (log level: info)
pnpm tauri dev

# Desarrollo con más detalle (log level: debug)
pnpm tauri:dev:debug

# Desarrollo con máximo detalle (log level: trace)
pnpm tauri:dev:trace

# Build de producción
pnpm tauri build
```

**Nota**: La primera compilación tarda varios minutos porque RocksDB (el motor de almacenamiento de PoloDB) se compila desde source. Las siguientes compilaciones son rápidas.

## Licencia

MIT
