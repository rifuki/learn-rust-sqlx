FROM mariadb:latest

COPY init.sql /docker-entrypoint-initdb.d/
RUN chmod 755 /docker-entrypoint-initdb.d/init.sql

EXPOSE ${DB_PORT}

COPY ./mariadb-data /var/lib/mysql

ENV MYSQL_ROOT_PASSWORD=${DB_ROOT_PASSWORD}
ENV MYSQL_USER=${DB_USER}
ENV MYSQL_PASSWORD=${DB_PASSWORD}
ENV MYSQL_DATABASE=${DB_NAME}