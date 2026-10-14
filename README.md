# mxchro

Automatizador de macros de teclado para Windows que permite grabar, configurar y ejecutar secuencias de teclas con diferentes modos de activación.

## Características

- **Múltiples modos de activación**: Once (una vez), Hold (mantener), Toggle (alternar)
- **Hook de teclado global**: Captura teclas a nivel del sistema
- **Simulación de teclado y ratón**: Ejecuta secuencias de teclas y clicks de ratón con timing preciso
- **Soporte completo de ratón**: Botones izquierdo, derecho, central, X1 y X2
- **Colecciones de macros**: Organiza macros en colecciones para mejor gestión
- **Toggle hotkeys globales**: Activa/desactiva macros individuales o colecciones completas desde cualquier aplicación
- **Sincronización en tiempo real**: La UI se actualiza automáticamente cuando cambias el estado con hotkeys (usando eventos de Tauri)
- **Control de macros**: Activar/desactivar macros en tiempo real
- **Bloqueo de teclas**: Opción de bloquear la tecla original
- **Filtrado de teclas inyectadas**: Evita loops infinitos

## Arquitectura del Backend

El backend está diseñado con una arquitectura en capas para maximizar la portabilidad y facilitar el mantenimiento.

```
src-tauri/src/
├── core/                          # Lógica de negocio pura
│   ├── types.rs                   # Tipos de dominio (Entity, View)
│   ├── managers/                  # Managers de dominio
│   │   ├── macro_manager.rs       # Gestión de macros y eventos
│   │   └── collection_manager.rs  # Gestión de colecciones
│   └── persistence/               # Persistencia con PoloDB (Document)
│       ├── macro_repository.rs
│       └── collection_repository.rs
├── infrastructure/                # Implementaciones de plataforma
│   └── input/
│       ├── hook.rs                # Hook de Windows (WH_KEYBOARD_LL)
│       ├── mapper.rs              # Mapeo de teclas VK ↔ String
│       └── simulator.rs           # InputSimulator: teclado y ratón (SendInput)
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

#### `managers/macro_manager.rs`

Servicio principal que gestiona el ciclo de vida de las macros:

- **Gestión de macros**: Agregar, eliminar, activar, desactivar, listar
- **Ejecución de macros**: Maneja los modos de activación (Once, Hold, Toggle)
- **Control de threads**: Usa `Condvar` para threads eficientes que se duermen/despiertan
- **Eventos de teclado**: `process_key_event()` procesa eventos del hook
- **Toggle hotkeys**: Maneja hotkeys globales para activar/desactivar macros y colecciones
- **Delegación**: Delega operaciones de colecciones a `CollectionManager`

**Estructuras principales**:

- **`MacroManager`**: Servicio público con toda la API
- **`MacroInstance`**: Instancia interna de una macro con su estado de ejecución
- **`RepeatState`**: Estado compartido para controlar threads de repetición (active, should_stop)
- **`ToggleEvent`** y **`ToggleTarget`**: Eventos de toggle para sincronización con frontend

#### `managers/collection_manager.rs`

Gestiona la lógica de colecciones de macros:

- **Gestión de colecciones**: Crear, eliminar, renombrar, reordenar
- **Toggle masivo**: Activar/desactivar todas las macros de una colección
- **Toggle hotkeys**: Hotkeys globales para controlar colecciones completas
- **Estado compartido**: Usa `Arc<RwLock<>>` para compartir estado con `MacroManager`

#### `types.rs`

Define los tipos de dominio siguiendo las convenciones de arquitectura limpia:

**Entities (Dominio puro):**
- **`Mode`**: Enum con los modos de activación (Toggle, Hold, Once)
- **`Action`**: Enum con las acciones posibles (Key o Wait)
- **`Macro`**: Configuración completa de una macro
- **`Collection`**: Grupo de macros con propiedades compartidas
- **`ActionCallback`**: Alias de tipo para closures que ejecutan acciones

**Views (Proyecciones para UI):**
- **`MacroView`**: Proyección parcial de una macro para listar (sin las acciones completas)
- **`CollectionView`**: Proyección de una colección con conteo de macros

### Capa Infrastructure (Plataforma)

La capa `infrastructure/` contiene implementaciones específicas de Windows.

#### `input/hook.rs`

Implementa el hook de teclado de bajo nivel de Windows:

- Instala un hook global `WH_KEYBOARD_LL`
- Captura todos los eventos de teclado del sistema
- Detecta teclas inyectadas (flag `LLKHF_INJECTED`)
- Notifica al `MacroManager` de cada evento
- Bloquea teclas si la macro lo requiere

#### `input/mapper.rs`

Mapeo bidireccional entre códigos virtuales de Windows (VK) y strings:

- Singleton con `Lazy<KeyMapper>` para eficiencia
- Soporta letras A-Z, dígitos 0-9, F1-F24, teclas especiales, numpad, mouse, multimedia
- Usa `IndexMap` para mantener orden de inserción
- Métodos: `string_to_vk()`, `vk_to_string()`, `strings_vec()`

#### `input/simulator.rs`

`InputSimulator`: Simulación de teclado y ratón usando la API moderna `SendInput`:

- Para teclado: envía teclas por scan code (más confiable que VK)
- Para ratón: usa `INPUT_MOUSE` con `MOUSEINPUT` para simular clicks
- Soporta botones de ratón: izquierdo, derecho, central, X1, X2
- Detecta automáticamente si el VK es teclado o ratón y usa la API correcta

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

1. **Inicio**: Se crean `MacroManager` y `CollectionManager`, se instalan el hook de teclado y el listener de eventos
2. **Usuario crea macro**: Frontend envía `Macro` vía comando Tauri
3. **MacroManager procesa**: Convierte strings a VK codes, construye acciones, spawnea thread si está enabled
4. **Hook captura tecla**: Notifica al `MacroManager` con `process_key_event()`
5. **Toggle hotkey detection**: Si la tecla es un toggle hotkey, se emite un evento de Tauri al frontend
6. **MacroManager ejecuta**: Según el modo (Once/Hold/Toggle), ejecuta la acción o activa el thread de repetición
7. **Thread de repetición**: Se despierta con `Condvar`, ejecuta acciones en loop, se duerme cuando se desactiva
8. **Simulación**: `InputSimulator::key_down()` envía las teclas o clicks al sistema
9. **Sincronización**: El frontend recibe el evento de Tauri y actualiza la UI automáticamente

## Portabilidad

La arquitectura en capas facilita la migración:

- **Cambiar framework** (Electron, GTK, etc.): Solo reescribir `adapters/` y `lib.rs`
- **Cambiar sistema operativo** (Linux, macOS): Solo reescribir `infrastructure/input/`
- **Core**: Se mantiene intacto en todos los casos

## Arquitectura del Frontend

El frontend está organizado siguiendo las mismas convenciones de nomenclatura:

```
src/
├── model/                         # Tipos de dominio (camelCase)
│   ├── Macro.ts                   # Entity: Macro
│   ├── Action.ts                  # Entity: Action
│   ├── Mode.ts                    # Entity: Mode
│   ├── MacroView.ts               # View: MacroView
│   ├── Collection.ts              # Entity: Collection
│   └── CollectionView.ts          # View: CollectionView
├── services/                      # Servicios y API
│   ├── MacroService.ts            # Conversión Response → Model (macros)
│   ├── CollectionService.ts       # Conversión Response → Model (colecciones)
│   └── KeyListService.ts          # Mapeo de teclas
├── contexts/                      # Contextos React
│   └── CollectionContext.tsx       # Estado global de colecciones
├── components/                    # Componentes UI
│   ├── ui/                        # Componentes shadcn/ui
│   ├── CollectionSidebar.tsx      # Sidebar de colecciones
│   ├── KeyInput.tsx               # Input para capturar teclas
│   └── ...
└── pages/                         # Páginas de la aplicación
    ├── home/
    │   └── HomePage.tsx           # Vista principal con sidebar de colecciones
    └── create_macro/
        └── CreateMacroPage.tsx    # Formulario de creación/edición
```

**Convención de conversión:**
```
Backend: Macro/Collection (snake_case en JSON)
    ↓
Frontend: MacroResponse/CollectionViewResponse (snake_case interface)
    ↓ fromSnakeCase()
Frontend: Macro/CollectionView (camelCase model para UI)
```

**Sistema de eventos:**
- Frontend escucha eventos de Tauri (`toggle-event`) para sincronización automática
- Cuando se usa un toggle hotkey, el backend emite un evento que el frontend recibe
- La UI se actualiza automáticamente sin necesidad de polling o refresh manual

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
  - `tailwind-animations`: Plugin de animaciones para Tailwind CSS
  - `uuid`: Generación de IDs únicos para colecciones

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


