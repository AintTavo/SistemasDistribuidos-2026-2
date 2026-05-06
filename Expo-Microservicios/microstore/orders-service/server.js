const express = require('express');
const http = require('http');
const Database = require('better-sqlite3');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 3002;
const DATA_DIR = '/app/data';
const DB_PATH = path.join(DATA_DIR, 'orders.db');

if (!fs.existsSync(DATA_DIR)) {
  fs.mkdirSync(DATA_DIR, { recursive: true });
}

console.log(`[ORDERS] Iniciando... DB: ${DB_PATH}`);

let db;
try {
  db = new Database(DB_PATH);
  console.log('[ORDERS] Base de datos abierta');
} catch (e) {
  console.error('[ORDERS] Error al abrir BD:', e.message);
  process.exit(1);
}

db.exec(`
  CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer TEXT NOT NULL,
    total REAL NOT NULL,
    status TEXT DEFAULT 'pendiente',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );
`);

db.exec(`
  CREATE TABLE IF NOT EXISTS order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    product_name TEXT NOT NULL,
    price REAL NOT NULL,
    quantity INTEGER NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders(id)
  );
`);
console.log('[ORDERS] Tablas verificadas/creadas');

const count = db.prepare('SELECT COUNT(*) as count FROM orders').get();
console.log(`[ORDERS] Pedidos en BD: ${count.count}`);

if (count.count === 0) {
  console.log('[ORDERS] Insertando pedido demo...');
  const insertOrder = db.prepare('INSERT INTO orders (customer, total, status) VALUES (?, ?, ?)');
  const result = insertOrder.run('Cliente Demo', 290, 'completado');
  const orderId = result.lastInsertRowid;
  
  const insertItem = db.prepare('INSERT INTO order_items (order_id, product_id, product_name, price, quantity) VALUES (?, ?, ?, ?, ?)');
  insertItem.run(orderId, 1, 'Hamburguesa Clásica', 120, 2);
  insertItem.run(orderId, 8, 'Refresco', 25, 2);
  console.log('[ORDERS] Pedido demo creado');
}

const cors = require('cors');
app.use(cors());
app.use(express.json());

app.use((req, res, next) => {
  console.log(`[ORDERS] ${req.method} ${req.path}`);
  next();
});

function fetchProduct(productId) {
  return new Promise((resolve, reject) => {
    console.log(`[ORDERS] Fetching product ${productId} from products-service`);
    const req = http.request(
      { hostname: 'products', port: 3001, path: `/products/${productId}`, method: 'GET', timeout: 5000 },
      (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try {
            console.log(`[ORDERS] Product ${productId} response: ${res.statusCode}`);
            resolve({ status: res.statusCode, data: JSON.parse(data) });
          } catch (e) {
            console.error(`[ORDERS] Parse error for product ${productId}:`, e.message);
            reject(e);
          }
        });
      }
    );
    req.on('error', (e) => {
      console.error(`[ORDERS] HTTP error for product ${productId}:`, e.message);
      reject(e);
    });
    req.on('timeout', () => { 
      req.destroy(); 
      reject(new Error('Timeout fetching product')); 
    });
    req.end();
  });
}

app.get('/', (req, res) => {
  res.json({ service: 'orders' });
});

app.get('/orders', (req, res) => {
  console.log('[ORDERS] GET /orders - Fetching all');
  const orders = db.prepare('SELECT * FROM orders ORDER BY id DESC').all();
  const items = db.prepare('SELECT * FROM order_items').all();
  const result = orders.map(o => ({
    ...o,
    items: items.filter(i => i.order_id === o.id).map(i => ({
      productId: i.product_id,
      product: { name: i.product_name, price: i.price },
      quantity: i.quantity
    }))
  }));
  console.log(`[ORDERS] Returning ${result.length} orders`);
  res.json(result);
});

app.get('/orders/:id', (req, res) => {
  const order = db.prepare('SELECT * FROM orders WHERE id = ?').get(parseInt(req.params.id));
  if (!order) return res.status(404).json({ error: 'Orden no encontrada' });
  const items = db.prepare('SELECT * FROM order_items WHERE order_id = ?').all(order.id);
  order.items = items.map(i => ({
    productId: i.product_id,
    product: { name: i.product_name, price: i.price },
    quantity: i.quantity
  }));
  res.json(order);
});

app.post('/orders', async (req, res) => {
  const { customer, items } = req.body;
  
  console.log(`[ORDERS] POST /orders - Customer: ${customer}, Items: ${JSON.stringify(items)}`);
  
  if (!customer || !items || !items.length) {
    console.log('[ORDERS] Invalid data');
    return res.status(400).json({ error: 'Datos inválidos' });
  }

  let total = 0;
  const validatedItems = [];

  for (const item of items) {
    try {
      console.log(`[ORDERS] Validating product ${item.productId}...`);
      const response = await fetchProduct(item.productId);
      
      if (response.status !== 200) {
        console.log(`[ORDERS] Product ${item.productId} not found (${response.status})`);
        return res.status(400).json({ error: `Producto ${item.productId} no encontrado` });
      }
      
      const product = response.data;
      const subtotal = product.price * item.quantity;
      total += subtotal;
      validatedItems.push({
        productId: item.productId,
        product,
        quantity: item.quantity,
        subtotal
      });
      console.log(`[ORDERS] Product ${product.name} valid: $${subtotal}`);
    } catch (e) {
      console.error(`[ORDERS] Error validating product ${item.productId}:`, e.message);
      return res.status(500).json({ error: `Error al validar producto ${item.productId}` });
    }
  }

  console.log(`[ORDERS] Total calculated: $${total}`);

  const result = db.prepare('INSERT INTO orders (customer, total, status) VALUES (?, ?, ?)').run(customer, total, 'pendiente');
  const orderId = result.lastInsertRowid;

  const insertItem = db.prepare('INSERT INTO order_items (order_id, product_id, product_name, price, quantity) VALUES (?, ?, ?, ?, ?)');
  for (const item of validatedItems) {
    insertItem.run(orderId, item.productId, item.product.name, item.product.price, item.quantity);
  }

  console.log(`[ORDERS] ✅ New order created: #${orderId} - ${customer} - $${total}`);

  res.status(201).json({
    id: orderId,
    customer,
    items: validatedItems,
    total,
    status: 'pendiente'
  });
});

app.patch('/orders/:id/status', (req, res) => {
  const { status } = req.body;
  const validStatuses = ['pendiente', 'preparando', 'completado', 'cancelado'];
  if (!validStatuses.includes(status)) {
    return res.status(400).json({ error: 'Status inválido' });
  }
  
  const result = db.prepare('UPDATE orders SET status = ? WHERE id = ?').run(status, parseInt(req.params.id));
  if (result.changes === 0) {
    return res.status(404).json({ error: 'Orden no encontrada' });
  }
  res.json({ success: true, status });
});

app.get('/health', (req, res) => {
  res.json({ status: 'ok', service: 'orders' });
});

app.listen(PORT, () => {
  console.log(`Orders Service running on port ${PORT}`);
});

process.on('SIGTERM', () => {
  console.log('[ORDERS] Closing...');
  db.close();
});