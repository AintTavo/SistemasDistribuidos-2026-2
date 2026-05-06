# 1. Imagen base (el sistema operativo/lenguaje)
FROM node:18-alpine

# 2. Directorio de trabajo dentro del contenedor
WORKDIR /app

# 3. Copiar archivos de dependencias
COPY package*.json ./

# 4. Instalar dependencias
RUN npm install

# 5. Copiar el resto del código
COPY . .

# 6. Puerto que usará la app
EXPOSE 3000

# 7. Comando para iniciar la app
CMD ["npm", "start"]