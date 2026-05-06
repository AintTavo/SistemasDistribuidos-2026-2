const express = require("express");
const { createProxyMiddleware } = require("http-proxy-middleware");
const cors = require("cors");
const morgan = require("morgan");

const app = express();
const PORT = process.env.PORT || 3000;

// URLs de los microservicios
const SERVICES = {
  users: process.env.USERS_SERVICE_URL || "http://localhost:8001",
  items: process.env.ITEMS_SERVICE_URL || "http://localhost:8002",
  purchases: process.env.PURCHASES_SERVICE_URL || "http://localhost:8003",
};

// Middlewares globales
app.use(cors());
app.use(morgan("combined"));
app.use(express.json());

// ─── Logging de peticiones entrantes ────────────────────────────────────────
app.use((req, _res, next) => {
  console.log(`[GATEWAY] ${req.method} ${req.path} → enrutando...`);
  next();
});

// ─── Rutas de health del gateway ────────────────────────────────────────────
app.get("/health", (_req, res) => {
  res.json({
    status: "ok",
    service: "api-gateway",
    uptime: process.uptime(),
    services: SERVICES,
  });
});

app.get("/", (_req, res) => {
  res.json({
    name: "MicroStore API Gateway",
    version: "1.0.0",
    routes: [
      { path: "/api/users/*", target: SERVICES.users },
      { path: "/api/items/*", target: SERVICES.items },
      { path: "/api/purchases/*", target: SERVICES.purchases },
    ],
  });
});

// ─── Proxy hacia Users Service ───────────────────────────────────────────────
// POST /api/users/signin  → POST {USERS_SERVICE}/users/signin
// GET  /api/users/login   → GET  {USERS_SERVICE}/users/login
// GET  /api/users/email/:email → GET {USERS_SERVICE}/users/email/:email
app.use(
  "/api/users",
  createProxyMiddleware({
    target: SERVICES.users,
    changeOrigin: true,
    pathRewrite: { "^/api/users": "/users" },
    on: {
      proxyReq: (proxyReq, req) => {
        console.log(`[GATEWAY → USERS] ${req.method} ${req.path}`);
      },
      error: (err, _req, res) => {
        console.error("[GATEWAY] Error en proxy de users:", err.message);
        res.status(502).json({ success: false, message: "Users Service no disponible" });
      },
    },
  })
);

// ─── Proxy hacia Items Service ───────────────────────────────────────────────
// GET /api/items?length=N → GET {ITEMS_SERVICE}/items?length=N
// GET /api/items/:id      → GET {ITEMS_SERVICE}/items/:id
app.use(
  "/api/items",
  createProxyMiddleware({
    target: SERVICES.items,
    changeOrigin: true,
    pathRewrite: { "^/api/items": "/items" },
    on: {
      proxyReq: (proxyReq, req) => {
        console.log(`[GATEWAY → ITEMS] ${req.method} ${req.path}`);
      },
      error: (err, _req, res) => {
        console.error("[GATEWAY] Error en proxy de items:", err.message);
        res.status(502).json({ success: false, message: "Items Service no disponible" });
      },
    },
  })
);

// ─── Proxy hacia Purchases Service ──────────────────────────────────────────
// POST /api/purchases      → POST {PURCHASES_SERVICE}/purchases
// GET  /api/purchases      → GET  {PURCHASES_SERVICE}/purchases
// GET  /api/purchases/:id  → GET  {PURCHASES_SERVICE}/purchases/:id
app.use(
  "/api/purchases",
  createProxyMiddleware({
    target: SERVICES.purchases,
    changeOrigin: true,
    pathRewrite: { "^/api/purchases": "/purchases" },
    on: {
      proxyReq: (proxyReq, req) => {
        console.log(`[GATEWAY → PURCHASES] ${req.method} ${req.path}`);
      },
      error: (err, _req, res) => {
        console.error("[GATEWAY] Error en proxy de purchases:", err.message);
        res.status(502).json({ success: false, message: "Purchases Service no disponible" });
      },
    },
  })
);

// ─── 404 handler ────────────────────────────────────────────────────────────
app.use((_req, res) => {
  res.status(404).json({ success: false, message: "Ruta no encontrada en el API Gateway" });
});

app.listen(PORT, () => {
  console.log(`\n🌐 API Gateway corriendo en http://0.0.0.0:${PORT}`);
  console.log(`   → /api/users     → ${SERVICES.users}`);
  console.log(`   → /api/items     → ${SERVICES.items}`);
  console.log(`   → /api/purchases → ${SERVICES.purchases}\n`);
});
