const express = require('express');
const cors = require('cors');
const { createProxyMiddleware } = require('http-proxy-middleware');

const app = express();
const PORT = process.env.PORT || 3003;

app.use(cors({
  origin: '*',
  methods: ['GET', 'POST', 'PATCH', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization']
}));
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true }));

app.use((req, res, next) => {
  console.log(`[GATEWAY] ${new Date().toISOString()} ${req.method} ${req.path} - Body: ${JSON.stringify(req.body)}`);
  next();
});

app.get('/health', (req, res) => res.json({ status: 'ok', service: 'gateway' }));

app.use('/api/products', createProxyMiddleware({
  target: 'http://products:3001',
  changeOrigin: true,
  pathRewrite: (path) => path.replace('/api/products', '/products'),
  onProxyReq: (proxyReq, req, res) => {
    console.log(`[GATEWAY -> PRODUCTS] ${req.method} /products`);
  },
  onError: (err, req, res) => {
    console.error('[GATEWAY ERROR] Products proxy error:', err.message);
    res.status(502).json({ error: 'Error connecting to products service' });
  }
}));

app.use('/api/orders', createProxyMiddleware({
  target: 'http://orders:3002',
  changeOrigin: true,
  pathRewrite: (path) => path.replace('/api/orders', '/orders'),
  onProxyReq: (proxyReq, req, res) => {
    console.log(`[GATEWAY -> ORDERS] ${req.method} /orders`);
  },
  onError: (err, req, res) => {
    console.error('[GATEWAY ERROR] Orders proxy error:', err.message);
    res.status(502).json({ error: 'Error connecting to orders service' });
  }
}));

app.use((req, res) => {
  res.status(404).json({ error: 'Ruta no encontrada', path: req.path });
});

app.listen(PORT, () => {
  console.log(`API Gateway running on port ${PORT}`);
});