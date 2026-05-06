const express = require('express');
const Database = require('better-sqlite3');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 3001;
const DATA_DIR = '/app/data';
const DB_PATH = path.join(DATA_DIR, 'products.db');

if (!fs.existsSync(DATA_DIR)) {
  fs.mkdirSync(DATA_DIR, { recursive: true });
}

console.log(`[PRODUCTS] Iniciando... DB: ${DB_PATH}`);

let db;
try {
  db = new Database(DB_PATH);
  console.log('[PRODUCTS] Base de datos abierta');
} catch (e) {
  console.error('[PRODUCTS] Error al abrir BD:', e.message);
  process.exit(1);
}

db.exec(`
  CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    price REAL NOT NULL,
    category TEXT NOT NULL
  );
`);
console.log('[PRODUCTS] Tabla products verificada/creada');

const count = db.prepare('SELECT COUNT(*) as count FROM products').get();
console.log(`[PRODUCTS] Productos en BD: ${count.count}`);

if (count.count === 0) {
  const insert = db.prepare('INSERT INTO products (name, price, category) VALUES (?, ?, ?)');
  const defaultProducts = [
    ['Hamburguesa Clásica', 120, 'Comida'],
    ['Hamburguesa con Queso', 140, 'Comida'],
    ['Pizza Margherita', 180, 'Comida'],
    ['Pizza Hawaiana', 200, 'Comida'],
    ['Ensalada Caesar', 90, 'Comida'],
    ['Hot Dog', 60, 'Comida'],
    ['Tacos al Pastor', 85, 'Comida'],
    ['Refresco', 25, 'Bebida'],
    ['Refresco Grande', 35, 'Bebida'],
    ['Cerveza', 45, 'Bebida'],
    ['Cerveza Artesanal', 65, 'Bebida'],
    ['Café', 35, 'Bebida'],
    ['Café con Leche', 45, 'Bebida'],
    ['Jugo Natural', 30, 'Bebida'],
    ['Agua Mineral', 20, 'Bebida'],
    ['Pastel de Chocolate', 60, 'Postre'],
    ['Helado', 40, 'Postre'],
    ['Flan', 35, 'Postre']
  ];
  defaultProducts.forEach(p => insert.run(...p));
  console.log('[PRODUCTS] Datos iniciales insertados');
}

const cors = require('cors');
app.use(cors());
app.use(express.json());

app.use((req, res, next) => {
  console.log(`[PRODUCTS] ${req.method} ${req.path}`);
  next();
});

app.get('/', (req, res) => {
  const products = db.prepare('SELECT id, name, price, category FROM products').all();
  res.json({ service: 'products', count: products.length, products });
});

app.get('/products', (req, res) => {
  console.log('[PRODUCTS] GET /products - Fetching all');
  const products = db.prepare('SELECT id, name, price, category FROM products').all();
  console.log(`[PRODUCTS] Returning ${products.length} products`);
  res.json(products);
});

app.get('/products/:id', (req, res) => {
  const id = parseInt(req.params.id);
  console.log(`[PRODUCTS] GET /products/${id}`);
  const product = db.prepare('SELECT id, name, price, category FROM products WHERE id = ?').get(id);
  if (!product) {
    console.log(`[PRODUCTS] Product ${id} not found`);
    return res.status(404).json({ error: 'Producto no encontrado' });
  }
  res.json(product);
});

app.post('/products', (req, res) => {
  console.log(`[PRODUCTS] POST /products - Body: ${JSON.stringify(req.body)}`);
  const { name, price, category } = req.body;
  if (!name || !price || !category) {
    console.log('[PRODUCTS] Missing fields');
    return res.status(400).json({ error: 'Datos incompletos', received: { name, price, category } });
  }
  try {
    const result = db.prepare('INSERT INTO products (name, price, category) VALUES (?, ?, ?)').run(name, price, category);
    const newProduct = { id: result.lastInsertRowid, name, price, category };
    console.log(`[PRODUCTS] Created: ${JSON.stringify(newProduct)}`);
    res.status(201).json(newProduct);
  } catch (e) {
    console.error('[PRODUCTS] Insert error:', e.message);
    res.status(500).json({ error: e.message });
  }
});

app.delete('/products/:id', (req, res) => {
  const result = db.prepare('DELETE FROM products WHERE id = ?').run(parseInt(req.params.id));
  if (result.changes === 0) {
    return res.status(404).json({ error: 'Producto no encontrado' });
  }
  res.json({ success: true });
});

app.get('/health', (req, res) => {
  res.json({ status: 'ok', service: 'products' });
});

app.listen(PORT, () => {
  console.log(`Products Service running on port ${PORT}`);
});

process.on('SIGTERM', () => {
  console.log('[PRODUCTS] Closing...');
  db.close();
});