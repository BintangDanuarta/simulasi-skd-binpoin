FROM node:22-alpine
WORKDIR /app
COPY package.json package-lock.json ./
COPY node_modules ./node_modules
COPY .next ./.next
COPY public ./public
EXPOSE 3000
CMD ["npm", "run", "start"]
