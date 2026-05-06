FROM node:18-alpine

RUN apk add --no-cache python3 make g++

WORKDIR /app

COPY package*.json ./

RUN npm install --production

RUN mkdir -p /app/data

COPY . .

EXPOSE 3001

CMD ["node", "server.js"]