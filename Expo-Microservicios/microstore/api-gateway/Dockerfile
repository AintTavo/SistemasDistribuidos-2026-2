FROM node:20-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY index.js ./

EXPOSE 3000

ENV USERS_SERVICE_URL=http://users-service:8001
ENV ITEMS_SERVICE_URL=http://items-service:8002
ENV PURCHASES_SERVICE_URL=http://purchases-service:8003

CMD ["node", "index.js"]
