FROM jdrouet/wasm-pack:lts-buster-20210103

WORKDIR /usr/share/wasm-julia

COPY . .

WORKDIR wasm-julia

RUN wasm-pack build

WORKDIR www/

RUN rm -rf ./node_modules/ \
    && npm install

ENV PORT=8080

CMD npm run start -- --port $PORT --public "wasm-julia.herokuapp.com"