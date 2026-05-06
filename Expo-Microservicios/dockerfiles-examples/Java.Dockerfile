# Etapa 1: Compilación
FROM maven:3.9-eclipse-temurin-17 AS build
WORKDIR /app
COPY . .
RUN mvn clean package -DskipTests

# Etapa 2: Ejecución (JRE ligero)
FROM eclipse-temurin:17-jre-alpine
WORKDIR /app

# El nombre del .jar depende de tu pom.xml
COPY --from=build /app/target/*.jar app.jar

EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]