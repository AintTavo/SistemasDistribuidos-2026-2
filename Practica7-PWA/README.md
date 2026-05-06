# Dungeon Crawler PWA — Práctica 7

Un dungeon crawler para web con backend en Rust/Axum y frontend PWA.

---

## Estructura del proyecto

```
dungeon/
├── backend/
│   ├── main.rs        ← API Rust corregida + lógica de items
│   ├── Cargo.toml
│   └── .env.example   ← Copiar a .env y completar
├── frontend/
│   ├── index.html     ← Shell de la PWA
│   ├── index.css      ← Estilos (basado en template, + temas por nivel)
│   ├── game.js        ← Toda la lógica del juego
│   ├── sw.js          ← Service Worker (Stale-While-Revalidate)
│   └── manifest.json  ← Manifiesto PWA
└── README.md
```

---

## Backend (Rust)

### Endpoints

| Método | Ruta      | Descripción                          |
|--------|-----------|--------------------------------------|
| POST   | `/login`  | Autenticar usuario → devuelve api_key |
| GET    | `/item`   | Obtener item aleatorio según nivel    |
| POST   | `/level`  | Actualizar nivel de sesión            |

### Configuración `.env`

```env
PORT=3000
API_USERS=admin,player1
PASSWORD=tu_contraseña
API_PASSWORD=tu_api_key_secreta
```

### Ejecutar

```bash
cp .env.example .env
# edita .env con tus credenciales
cargo run --release
```

### Correcciones aplicadas a `main.rs` original

- `HeshMap` → `HashMap` (typo)  
- Struct `PetitionGet` duplicada (conflicto con `ResponseGet`) → renombradas y separadas  
- Handlers incompletos → implementados con lógica real  
- CORS agregado con `tower-http`  
- Pool de items con **pesos**: armas ×3, pociones ×2, magias ×1 (más raras)  
- Estado de sesiones en `Arc<Mutex<HashMap>>`  

---

## Frontend (PWA)

### Mecánicas principales

**Grupos de enemigos**
- 1 enemigo → tier 1 (fuerte, mucho HP y defensa)  
- 2 enemigos → tier 2 (intermedios)  
- 3 enemigos → tier 3 (débiles)  
- ≥3 tipos de enemigo por dificultad (nivel)

**Niveles y escalado**
- Cada 3 grupos eliminados → sube 1 nivel de mazmorra  
- El nivel escala HP y defensa de enemigos (+35% HP, +20% DEF por nivel)  
- La variable de nivel se comunica con la API vía `POST /level`  
- El color de los detalles cambia por nivel:
  - Nivel 1 → Verde (#c8f562)  
  - Nivel 2 → Azul (#62c8f5)  
  - Nivel 3 → Naranja (#f5a362)  
  - Nivel 4 → Morado (#c862f5)  
  - Nivel 5 → Dorado (#f5e262)  

**Items (vía API)**
- Armas: cambiar por la actual o descartar  
- Pociones: usar ahora, guardar (máx 3) o descartar. 3 tiers de cura  
- Magias: se aprenden permanentemente (máx 3), se conservan al morir. Más raras por peso  

**Sin conexión**
- Badge "● SIN CONEXIÓN" en esquina superior derecha  
- El juego funciona normalmente pero sin recompensas al ganar grupos  

**Persistencia**
- `localStorage` guarda: arma, pociones, nivel, estadísticas, URL/key de API  
- `localStorage` guarda las magias por separado (sobreviven a la muerte)  

### Service Worker — Stale-While-Revalidate

1. Sirve assets desde cache inmediatamente (stale)  
2. Actualiza cache en background con la red  
3. Si no hay cache aún → espera la red  
4. Offline: sirve `index.html` para navegación  
5. Las peticiones a `/login`, `/item`, `/level` **no** se cachean  

### Iconos

Crea la carpeta `icons/` con:
- `icon-192.png` (192×192)
- `icon-512.png` (512×512)

Puedes generarlos desde cualquier favicon generator online.

---

## Cómo probar localmente

```bash
# Terminal 1: backend
cd backend
cargo run

# Terminal 2: frontend (cualquier servidor estático)
cd frontend
npx serve .
# o: python3 -m http.server 8080
```

Luego abre `http://localhost:8080`, ve a la pestaña **CONEXIÓN** y autentícate con tus credenciales de `.env`.