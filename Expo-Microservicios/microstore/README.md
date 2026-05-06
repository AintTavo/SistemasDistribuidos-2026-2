# 🛒 MicroStore — Arquitectura de Microservicios

Tienda online construida con microservicios que se comunican de forma **síncrona** (HTTP REST). Los servicios de negocio están escritos en **Rust** (Actix-Web), el gateway en **Node.js** (Express), y el frontend en HTML + **Tailwind CSS**.

---

## 📐 Arquitectura

```
                        ┌─────────────────────────────┐
                        │        FRONTEND (Nginx:80)   │
                        │      HTML + Tailwind CSS     │
                        └─────────────┬───────────────┘
                                      │ /api/*
                        ┌─────────────▼───────────────┐
                        │    API GATEWAY (Node:3000)   │
                        │         Express.js           │
                        └──┬──────────┬────────────┬──┘
                           │          │            │
               ┌───────────▼──┐ ┌─────▼──────┐ ┌──▼────────────┐
               │ USERS        │ │ ITEMS      │ │ PURCHASES     │
               │ Rust :8001   │ │ Rust :8002 │ │ Rust :8003    │
               │              │ │            │ │               │
               │ POST /signin │ │ GET /items │ │ POST /        │
               │ GET /login   │ │ GET /:id   │ │ GET /         │
               │ GET /email/: │ │            │ │ ← llama a     │
               └──────────────┘ └────────────┘ │   users e     │
                        ▲                ▲      │   items sync  │
                        └────────────────┴──────┘
```

### Flujo de comunicación síncrona en Compras

```
Cliente → Gateway → purchases-service
                         │
                         ├──(GET /users/email/:email)──→ users-service
                         │                                   └─ responde con datos del usuario
                         │
                         ├──(GET /items/:id) ×N ──────→ items-service
                         │                                   └─ responde con datos de cada item
                         │
                         └── Crea registro con ID único → responde al cliente
```

---

## 📁 Estructura del proyecto

```
microstore/
├── users-service/          # Microservicio de usuarios (Rust)
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── Dockerfile
├── items-service/          # Microservicio de items (Rust)
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── Dockerfile
├── purchases-service/      # Microservicio de compras (Rust)
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── Dockerfile
├── api-gateway/            # API Gateway (Node.js)
│   ├── index.js
│   ├── package.json
│   └── Dockerfile
├── frontend/               # Frontend (HTML + Tailwind)
│   ├── index.html
│   ├── nginx.conf
│   └── Dockerfile
└── docker-compose.yml      # Orquestación completa
```

---

## 🚀 Inicio rápido (Docker Compose — RECOMENDADO)

La forma más sencilla de levantar todo el sistema:

```bash
# 1. Clonar/entrar al proyecto
cd microstore

# 2. Levantar todos los servicios
docker compose up --build

# 3. Acceder
# Frontend:  http://localhost
# Gateway:   http://localhost:3000
# Users:     http://localhost:8001
# Items:     http://localhost:8002
# Purchases: http://localhost:8003
```

Para correr en segundo plano:
```bash
docker compose up --build -d
```

Para ver logs de un servicio específico:
```bash
docker compose logs -f purchases-service
```

Para detener todo:
```bash
docker compose down
```

---

## 🐳 Contenedores individuales

Si deseas construir y correr cada microservicio por separado (sin Compose):

### Paso 0 — Crear la red compartida

```bash
docker network create microstore-net
```

---

### 1. Users Service (Puerto 8001)

```bash
cd users-service

# Construir imagen
docker build -t ms-users .

# Correr contenedor
docker run -d \
  --name ms-users \
  --network microstore-net \
  -p 8001:8001 \
  ms-users
```

**Verificar:**
```bash
curl http://localhost:8001/users/health
```

---

### 2. Items Service (Puerto 8002)

```bash
cd items-service

docker build -t ms-items .

docker run -d \
  --name ms-items \
  --network microstore-net \
  -p 8002:8002 \
  ms-items
```

**Verificar:**
```bash
curl "http://localhost:8002/items?length=3"
```

---

### 3. Purchases Service (Puerto 8003)

> ⚠️ **Importante:** Este servicio depende de users e items. Deben estar corriendo antes.

```bash
cd purchases-service

docker build -t ms-purchases .

docker run -d \
  --name ms-purchases \
  --network microstore-net \
  -p 8003:8003 \
  -e USERS_SERVICE_URL=http://ms-users:8001 \
  -e ITEMS_SERVICE_URL=http://ms-items:8002 \
  ms-purchases
```

> Las variables de entorno `USERS_SERVICE_URL` e `ITEMS_SERVICE_URL` permiten al servicio de compras comunicarse con los otros dos dentro de la red Docker usando sus nombres de contenedor.

**Verificar:**
```bash
curl http://localhost:8003/purchases/health
```

---

### 4. API Gateway (Puerto 3000)

```bash
cd api-gateway

docker build -t ms-gateway .

docker run -d \
  --name ms-gateway \
  --network microstore-net \
  -p 3000:3000 \
  -e USERS_SERVICE_URL=http://ms-users:8001 \
  -e ITEMS_SERVICE_URL=http://ms-items:8002 \
  -e PURCHASES_SERVICE_URL=http://ms-purchases:8003 \
  ms-gateway
```

**Verificar:**
```bash
curl http://localhost:3000/health
```

---

### 5. Frontend (Puerto 80)

```bash
cd frontend

docker build -t ms-frontend .

docker run -d \
  --name ms-frontend \
  --network microstore-net \
  -p 80:80 \
  ms-frontend
```

**Abrir:** http://localhost

---

## 🔌 API Reference

### Users Service — `/users`

| Método | Ruta | Descripción |
|--------|------|-------------|
| `POST` | `/users/signin` | Registrar nuevo usuario |
| `GET` | `/users/login?email=&password=` | Autenticar usuario |
| `GET` | `/users/email/:email` | Buscar por email (uso interno) |
| `GET` | `/users/health` | Health check |

**POST /users/signin**
```json
{
  "name": "Ana López",
  "email": "ana@ejemplo.com",
  "password": "mipassword"
}
```

**GET /users/login**
```
GET /users/login?email=alice@microstore.com&password=password123
```

---

### Items Service — `/items`

| Método | Ruta | Descripción |
|--------|------|-------------|
| `GET` | `/items?length=N` | Obtener N items |
| `GET` | `/items/:id` | Obtener item por ID |
| `GET` | `/items/health` | Health check |

```bash
# Obtener 5 items
curl "http://localhost:8002/items?length=5"

# Obtener item específico
curl "http://localhost:8002/items/item-001"
```

---

### Purchases Service — `/purchases`

| Método | Ruta | Descripción |
|--------|------|-------------|
| `POST` | `/purchases` | Crear compra (llama a users e items síncronamente) |
| `GET` | `/purchases` | Listar todas las compras |
| `GET` | `/purchases/:id` | Obtener compra por ID |
| `GET` | `/purchases/health` | Health check |

**POST /purchases**
```json
{
  "user_email": "alice@microstore.com",
  "item_ids": ["item-001", "item-003", "item-007"]
}
```

---

### API Gateway — Rutas públicas

Todas las rutas anteriores son accesibles a través del gateway con el prefijo `/api`:

```
GET  http://localhost:3000/api/items?length=5
POST http://localhost:3000/api/users/signin
GET  http://localhost:3000/api/users/login?email=...&password=...
POST http://localhost:3000/api/purchases
GET  http://localhost:3000/api/purchases
```

---

## 👤 Usuarios dummy

Disponibles por defecto (sin necesidad de registro):

| Nombre | Email | Contraseña |
|--------|-------|------------|
| Alice García | alice@microstore.com | password123 |
| Bob Martínez | bob@microstore.com | password123 |

---

## 🛍️ Items disponibles

| ID | Nombre | Precio |
|----|--------|--------|
| item-001 | Laptop Pro X | $24,999 |
| item-002 | Auriculares Quantum | $3,499 |
| item-003 | Smartwatch Nexus | $5,999 |
| item-004 | Teclado Mecánico RGB | $2,199 |
| item-005 | Monitor UltraWide 34" | $18,500 |
| item-006 | SSD NVMe 2TB | $1,899 |
| item-007 | Mouse Inalámbrico Pro | $1,299 |
| item-008 | Webcam 4K Stream | $2,799 |
| item-009 | Hub USB-C 12 en 1 | $899 |
| item-010 | Tablet Creator 12 | $12,999 |

---

## 🧪 Prueba rápida de flujo completo

```bash
# 1. Registrar usuario
curl -X POST http://localhost:3000/api/users/signin \
  -H "Content-Type: application/json" \
  -d '{"name":"Test User","email":"test@test.com","password":"123456"}'

# 2. Login
curl "http://localhost:3000/api/users/login?email=test@test.com&password=123456"

# 3. Ver items
curl "http://localhost:3000/api/items?length=3"

# 4. Crear compra (purchases llama síncronamente a users e items)
curl -X POST http://localhost:3000/api/purchases \
  -H "Content-Type: application/json" \
  -d '{"user_email":"test@test.com","item_ids":["item-001","item-004"]}'

# 5. Ver compras
curl http://localhost:3000/api/purchases
```

---

## ⚙️ Variables de entorno

| Variable | Servicio | Default | Descripción |
|----------|----------|---------|-------------|
| `USERS_SERVICE_URL` | purchases, gateway | http://localhost:8001 | URL del users-service |
| `ITEMS_SERVICE_URL` | purchases, gateway | http://localhost:8002 | URL del items-service |
| `PURCHASES_SERVICE_URL` | gateway | http://localhost:8003 | URL del purchases-service |
| `PORT` | gateway | 3000 | Puerto del API Gateway |

---

## 🔧 Desarrollo local (sin Docker)

Para correr los servicios Rust sin Docker necesitas tener instalado [Rust](https://rustup.rs/):

```bash
# Terminal 1 - Users
cd users-service && cargo run

# Terminal 2 - Items
cd items-service && cargo run

# Terminal 3 - Purchases
cd purchases-service && cargo run

# Terminal 4 - Gateway
cd api-gateway && node index.js

# Terminal 5 - Frontend (cualquier servidor estático)
cd frontend && npx serve .
```

---

## 📝 Notas de diseño

- **Sin base de datos persistente:** Los datos viven en memoria (`Lazy<Mutex<Vec<T>>>`). Al reiniciar un contenedor los datos se pierden (excepto los usuarios/items dummy que se inicializan al arrancar).
- **Comunicación síncrona:** El servicio de compras realiza peticiones HTTP directas a users e items en el momento de crear una compra. Si alguno falla, devuelve error 502.
- **Multi-stage Docker:** Los Dockerfiles usan compilación en dos etapas para producir imágenes mínimas (~50MB vs ~1GB en modo debug).
- **CORS permisivo:** Activado en todos los servicios para facilitar desarrollo. En producción se debe restringir.
