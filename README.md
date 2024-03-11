# Em Rust porque C é ilegal agora :(

> Veja uma versão mais interessante em C [aqui](https://github.com/felipemarkson/rinha-backend-q1-2024).

Basicamente tudo é Rust nesse repositório.

Tentei evitar ao máximo as depedências, porém essa não parece ser a filosofia de um software em Rust.

As quatro dependências podem ser vistas em [Cargo.toml](Cargo.toml).

A arquitetura é bem simples:

- `httpserver` é o load balancer que distribuí uma requisição para cada `api` por vez (round robin)
- `backend` é a `api`, onde as requisões são tratadas
- `database` é o banco de dados baseado em arquivos binários.

Com exceção do `httpserver` na porta `9999`, toda a comunicação é feita em `UDP` ao invés de `TCP` devido a velocidade e porque não tem perda de dados em uma conexão local :)

```
                           | <-> backend1 <-> |
req/res <-> httpserver <-> |                  | <-> database
                           | <-> backend2 <-> |
```